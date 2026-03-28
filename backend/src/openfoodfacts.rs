use crate::models::ProductInfo;
use serde_json::Value;
use std::env;

const OPENFOODFACTS_API_BASE: &str = "https://world.openfoodfacts.org";

/// Get User-Agent string for API requests
/// Format: AppName/Version (ContactEmail) as required by OpenFoodFacts API
/// See: https://openfoodfacts.github.io/openfoodfacts-server/api/
fn get_user_agent() -> String {
    std::env::var("OFF_USER_AGENT").unwrap_or_else(|_| {
        "Inventary/0.1.0 (inventary@example.com)".to_string()
    })
}

/// Create an HTTP client with proper User-Agent header as required by OpenFoodFacts API
/// Documentation: https://openfoodfacts.github.io/openfoodfacts-server/api/
fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(get_user_agent())
        .build()
        .expect("Failed to create HTTP client")
}

/// Get product information by barcode using OpenFoodFacts API v2
/// Documentation: https://openfoodfacts.github.io/openfoodfacts-server/api/
pub async fn get_product_by_barcode(
    barcode: &str,
    language: Option<&str>,
) -> Result<ProductInfo, Box<dyn std::error::Error>> {
    let lang = language.unwrap_or("en");
    let url = format!(
        "{}/api/v2/product/{}.json?lc={}",
        OPENFOODFACTS_API_BASE, barcode, lang
    );
    let client = create_client();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let json: Value = response.json().await?;

    // API v2 returns status: 0 if product not found, 1 if found
    if json["status"].as_i64() != Some(1) {
        return Err("Product not found".into());
    }

    let product = &json["product"];

    // Get product name (prefer translated, fallback to product_name, then others)
    let lang_key = format!("product_name_{}", lang);
    let name = product[&lang_key]
        .as_str()
        .or_else(|| product["product_name"].as_str())
        .or_else(|| product["product_name_en"].as_str())
        .or_else(|| product["product_name_fr"].as_str())
        .unwrap_or("Unknown Product")
        .to_string();

    // Get image URL (prefer front image, fallback to image_url)
    let image_url = product["image_front_url"]
        .as_str()
        .or_else(|| product["image_url"].as_str())
        .or_else(|| product["image_small_url"].as_str())
        .map(|s| format!("{}{}", OPENFOODFACTS_API_BASE, s))
        .or_else(|| product["image_url"].as_str().map(|s| s.to_string()));

    // Get brand (can be string or array)
    let brand = product["brands"]
        .as_str()
        .or_else(|| {
            product["brands_tags"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
        })
        .map(|s| s.to_string());

    // Get categories (standardized tags are preferred for localization)
    let lang_prefix = format!("{}:", lang);
    let mut categories = Vec::new();

    if let Some(cats_array) = product["categories_tags"].as_array() {
        // 1. Try to find tags in the requested language
        let mut lang_cats: Vec<String> = cats_array
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| s.starts_with(&lang_prefix))
            .map(|s| {
                let name = s.replace(&lang_prefix, "").replace('-', " ");
                // Title case
                let mut c = name.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => {
                        f.to_uppercase().collect::<String>() + c.as_str()
                    }
                }
            })
            .collect();

        if !lang_cats.is_empty() {
            categories = lang_cats;
        } else {
            // 2. Fallback to all standardized tags (cleaning them up)
            categories = cats_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| {
                    let name = if s.contains(':') {
                        s.split(':').nth(1).unwrap_or(s)
                    } else {
                        s
                    }
                    .replace('-', " ");

                    let mut c = name.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => {
                            f.to_uppercase().collect::<String>() + c.as_str()
                        }
                    }
                })
                .collect();
        }
    }

    // 3. Last fallback to raw categories string if no tags found
    if categories.is_empty() {
        if let Some(cats_str) = product["categories"].as_str() {
            categories = cats_str
                .split(',')
                .map(|cat| cat.trim().to_string())
                .filter(|cat| !cat.is_empty())
                .collect();
        }
    }

    Ok(ProductInfo {
        id: None,
        barcode: Some(barcode.to_string()),
        name,
        image_url,
        brand,
        categories,
        unit: None,
    })
}

/// Search for products using OpenFoodFacts API v2
/// Documentation: https://openfoodfacts.github.io/openfoodfacts-server/api/
/// Note: Rate limit is 10 requests per minute for search queries
pub async fn search_products(
    query: &str,
    language: Option<&str>,
) -> Result<Vec<ProductInfo>, Box<dyn std::error::Error>> {
    let lang = language.unwrap_or("en");
    // Use the /cgi/search.pl endpoint which is the standard for text search
    let url = format!(
        "{}/cgi/search.pl?search_terms={}&search_simple=1&action=process&json=1&page_size=20&lc={}",
        OPENFOODFACTS_API_BASE,
        urlencoding::encode(query),
        lang
    );

    let client = create_client();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let json: Value = response.json().await?;
    let empty_vec = Vec::new();
    let products = json["products"].as_array().unwrap_or(&empty_vec);

    let mut results = Vec::new();
    for product in products {
        // Get product name
        let lang_key = format!("product_name_{}", lang);
        let name = product[&lang_key]
            .as_str()
            .or_else(|| product["product_name"].as_str())
            .or_else(|| product["product_name_en"].as_str())
            .or_else(|| product["product_name_fr"].as_str())
            .unwrap_or("Unknown Product")
            .to_string();

        // Get barcode (code field)
        let barcode = product["code"]
            .as_str()
            .or_else(|| product["_id"].as_str())
            .map(|s| s.to_string());

        // Get image URL
        let image_url = product["image_front_url"]
            .as_str()
            .or_else(|| product["image_url"].as_str())
            .or_else(|| product["image_small_url"].as_str())
            .map(|s| {
                if s.starts_with("http") {
                    s.to_string()
                } else {
                    format!("{}{}", OPENFOODFACTS_API_BASE, s)
                }
            });

        // Get brand
        let brand = product["brands"]
            .as_str()
            .or_else(|| {
                product["brands_tags"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
            })
            .map(|s| s.to_string());

        // Get categories (standardized tags are preferred for localization)
        let lang_prefix = format!("{}:", lang);
        let mut categories = Vec::new();

        if let Some(cats_array) = product["categories_tags"].as_array() {
            // 1. Try to find tags in the requested language
            let mut lang_cats: Vec<String> = cats_array
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|s| s.starts_with(&lang_prefix))
                .map(|s| {
                    let name = s.replace(&lang_prefix, "").replace('-', " ");
                    // Title case
                    let mut c = name.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => {
                            f.to_uppercase().collect::<String>() + c.as_str()
                        }
                    }
                })
                .collect();

            if !lang_cats.is_empty() {
                categories = lang_cats;
            } else {
                // 2. Fallback to all standardized tags (cleaning them up)
                categories = cats_array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| {
                        let name = if s.contains(':') {
                            s.split(':').nth(1).unwrap_or(s)
                        } else {
                            s
                        }
                        .replace('-', " ");

                        let mut c = name.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => {
                                f.to_uppercase().collect::<String>()
                                    + c.as_str()
                            }
                        }
                    })
                    .collect();
            }
        }

        // 3. Last fallback to raw categories string if no tags found
        if categories.is_empty() {
            if let Some(cats_str) = product["categories"].as_str() {
                categories = cats_str
                    .split(',')
                    .map(|cat| cat.trim().to_string())
                    .filter(|cat| !cat.is_empty())
                    .collect();
            }
        }

        results.push(ProductInfo {
            id: None,
            barcode,
            name,
            image_url,
            brand,
            categories,
            unit: None,
        });
    }

    Ok(results)
}

/// Contribute a product to OpenFoodFacts
/// Documentation: https://openfoodfacts.github.io/openfoodfacts-server/api/tutorial-off-api/
pub async fn contribute_product(
    barcode: &str,
    name: &str,
    brand: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let username =
        env::var("OFF_USERNAME").map_err(|_| "OFF_USERNAME not set")?;
    let password =
        env::var("OFF_PASSWORD").map_err(|_| "OFF_PASSWORD not set")?;

    let url = format!("{}/cgi/product_jqm2.pl", OPENFOODFACTS_API_BASE);

    let mut params = std::collections::HashMap::new();
    params.insert("code", barcode.to_string());
    params.insert("product_name", name.to_string());
    params.insert("user_id", username);
    params.insert("password", password);

    if let Some(b) = brand {
        params.insert("brands", b.to_string());
    }

    let client = create_client();
    let response = client.post(&url).form(&params).send().await?;

    if !response.status().is_success() {
        return Err(
            format!("OFF contribution failed: {}", response.status()).into()
        );
    }

    Ok(())
}

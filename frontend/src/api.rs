use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const API_BASE: &str = "http://127.0.0.1:8080/api";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventoryItem {
    pub id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub product_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddItemRequest {
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveItemRequest {
    pub barcode: Option<String>,
    pub id: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductInfo {
    pub barcode: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub brand: Option<String>,
    pub categories: Vec<String>,
}

pub async fn fetch_inventory() -> Result<Vec<InventoryItem>, String> {
    let url = format!("{}/inventory", API_BASE);
    fetch_json(&url, None::<&()>).await
}

pub async fn add_item(req: AddItemRequest) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/add", API_BASE);
    fetch_json(&url, Some(&req)).await
}

pub async fn remove_item(req: RemoveItemRequest) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/remove", API_BASE);
    fetch_json(&url, Some(&req)).await
}

pub async fn search_products(query: &str) -> Result<Vec<ProductInfo>, String> {
    let url = format!("{}/search?q={}", API_BASE, urlencoding::encode(query));
    fetch_json(&url, None::<&()>).await
}

pub async fn search_inventory_items(query: &str) -> Result<Vec<ProductInfo>, String> {
    let url = format!("{}/inventory/search?q={}", API_BASE, urlencoding::encode(query));
    fetch_json(&url, None::<&()>).await
}

pub async fn get_product_by_barcode(barcode: &str) -> Result<ProductInfo, String> {
    let url = format!("{}/product/{}", API_BASE, barcode);
    fetch_json(&url, None::<&()>).await
}

async fn fetch_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    let mut opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
    if let Some(body_data) = body {
        opts.set_method("POST");
        let body_str = serde_json::to_string(body_data).map_err(|e| e.to_string())?;
        let body_js = JsValue::from_str(&body_str);
        opts.set_body(&body_js);
        let headers = web_sys::Headers::new().unwrap();
        headers.set("Content-Type", "application/json").unwrap();
        let headers_js: JsValue = headers.into();
        opts.set_headers(&headers_js);
    }
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;
    
    let resp: Response = resp_value.dyn_into()
        .map_err(|e| format!("Response is not a Response: {:?}", e))?;
    
    if !resp.ok() {
        let status = resp.status();
        // Try to parse error message from JSON body
        if let Ok(promise) = resp.json() {
            if let Ok(json) = JsFuture::from(promise).await {
                if let Ok(error_val) = js_sys::Reflect::get(&json, &JsValue::from_str("error")) {
                    if let Some(msg) = error_val.as_string() {
                        return Err(msg);
                    }
                }
            }
        }
        return Err(format!("HTTP error: {}", status));
    }
    
    let json = JsFuture::from(resp.json().map_err(|e| format!("Failed to get JSON: {:?}", e))?)
        .await
        .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;
    
    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Failed to deserialize: {:?}", e))
}

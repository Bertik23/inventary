use actix_web::{web, Result, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use crate::models::*;
use crate::schema::inventory_items;
use crate::openfoodfacts;
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde_json;
use bcrypt::{hash, verify, DEFAULT_COST};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

fn send_reset_email(to_email: &str, token: &str) -> Result<(), String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "localhost".to_string());
    let smtp_user = env::var("SMTP_USER").unwrap_or_else(|_| "".to_string());
    let smtp_pass = env::var("SMTP_PASS").unwrap_or_else(|_| "".to_string());
    let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse::<u16>().unwrap_or(587);
    let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@example.com".to_string());
    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());

    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Password Reset Request")
        .body(format!(
            "To reset your password, click the following link: {}/reset-password?token={}\n\nThis link will expire in 1 hour.",
            app_url, token
        ))
        .map_err(|e| e.to_string())?;

    if smtp_user.is_empty() {
        println!("SMTP_USER not set, printing reset email to console:");
        println!("To: {}", to_email);
        println!("Subject: Password Reset Request");
        println!("Body: To reset your password, click the following link: {}/reset-password?token={}", app_url, token);
        return Ok(());
    }

    let creds = Credentials::new(smtp_user, smtp_pass);

    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Could not send email: {:?}", e)),
    }
}

#[actix_web::get("/api/inventory")]
pub async fn show_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let inventory_id_param = query.get("inventory_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("inventory_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::inventory_items::dsl::inventory_id;
    let items = inventory_items::table
        .filter(inventory_id.eq(inventory_id_param))
        .load::<InventoryItem>(&mut conn)
        .map_err(|e| {
            eprintln!("Error loading inventory: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    
    Ok(HttpResponse::Ok().json(items))
}

#[actix_web::post("/api/inventory/add")]
pub async fn add_item(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<AddItemRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let product_info = if let Some(ref barcode_val) = req.barcode {
        match openfoodfacts::get_product_by_barcode(barcode_val).await {
            Ok(info) => Some(info),
            Err(_) => None,
        }
    } else {
        None
    };
    
    let item_name = if let Some(ref name_val) = req.name {
        name_val.clone()
    } else if let Some(ref info) = product_info {
        info.name.clone()
    } else {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Product name or barcode required"
        })));
    };
    
    // 1. Check if item already exists in this inventory (by barcode or name)
    let existing_item = if let Some(ref barcode_val) = req.barcode {
        use crate::schema::inventory_items::dsl::{barcode, inventory_id};
        inventory_items::table
            .filter(barcode.eq(barcode_val))
            .filter(inventory_id.eq(&req.inventory_id))
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else {
        use crate::schema::inventory_items::dsl::{name, inventory_id};
        inventory_items::table
            .filter(name.eq(&item_name))
            .filter(inventory_id.eq(&req.inventory_id))
            .first::<InventoryItem>(&mut conn)
            .ok()
    };

    // 2. Determine the unit to use
    let unit_val = if let Some(ref item) = existing_item {
        // Use existing item's unit
        item.unit.clone()
    } else {
        // Check for template/config
        use crate::schema::custom_item_templates::dsl::*;
        let template = custom_item_templates
            .filter(inventory_id.eq(&req.inventory_id).or(inventory_id.is_null()))
            .filter(name.eq(&item_name))
            .order(inventory_id.desc()) // Inventory-specific first (non-null > null)
            .first::<CustomItemTemplate>(&mut conn)
            .ok();
            
        if let Some(t) = template {
            t.default_unit
        } else {
            req.unit.clone().unwrap_or_else(|| "pcs".to_string())
        }
    };

    let qty = req.quantity.unwrap_or(1.0);
    let prod_data = product_info.as_ref()
        .and_then(|_| serde_json::to_string(product_info.as_ref().unwrap()).ok());
    
    if let Some(mut item) = existing_item {
        // Update existing item
        use crate::schema::inventory_items::dsl::{quantity, updated_at};
        diesel::update(inventory_items::table.find(&item.id))
            .set((
                quantity.eq(item.quantity + qty),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Error updating item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        item.quantity += qty;
        Ok(HttpResponse::Ok().json(item))
    } else {
        // Create new item
        let new_item = NewInventoryItem {
            id: Uuid::new_v4().to_string(),
            inventory_id: req.inventory_id.clone(),
            barcode: req.barcode.clone(),
            name: item_name,
            quantity: qty,
            unit: unit_val,
            product_data: prod_data,
        };
        
        diesel::insert_into(inventory_items::table)
            .values(&new_item)
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Error inserting item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        use crate::schema::inventory_items::dsl::id;
        let created_item = inventory_items::table
            .filter(id.eq(&new_item.id))
            .first::<InventoryItem>(&mut conn)
            .map_err(|e| {
                eprintln!("Error loading created item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Created().json(created_item))
    }
}

#[actix_web::post("/api/inventory/remove")]
pub async fn remove_item(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<RemoveItemRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let item = if let Some(ref item_id) = req.id {
        inventory_items::table
            .find(item_id)
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else if let Some(ref barcode_val) = req.barcode {
        use crate::schema::inventory_items::dsl::{barcode, inventory_id};
        inventory_items::table
            .filter(barcode.eq(barcode_val))
            .filter(inventory_id.eq(&req.inventory_id))
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else if let Some(ref name_val) = req.name {
        use crate::schema::inventory_items::dsl::{name, inventory_id};
        inventory_items::table
            .filter(name.eq(name_val))
            .filter(inventory_id.eq(&req.inventory_id))
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Item ID, barcode or name required"
        })));
    };
    
    let item = match item {
        Some(i) => i,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Item not found"
            })));
        }
    };
    
    let remove_quantity = req.quantity.unwrap_or(1.0);
    
    if item.quantity <= remove_quantity {
        // Remove item completely
        diesel::delete(inventory_items::table.find(&item.id))
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Error deleting item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Ok().json(item))
    } else {
        // Decrease quantity
        use crate::schema::inventory_items::dsl::{quantity, updated_at};
        diesel::update(inventory_items::table.find(&item.id))
            .set((
                quantity.eq(item.quantity - remove_quantity),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Error updating item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        let updated_item = inventory_items::table
            .find(&item.id)
            .first::<InventoryItem>(&mut conn)
            .map_err(|e| {
                eprintln!("Error loading updated item: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Ok().json(updated_item))
    }
}

#[actix_web::get("/api/inventory/templates")]
pub async fn get_custom_item_templates(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let inventory_id_param = query.get("inventory_id");
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::custom_item_templates::dsl::*;
    
    let mut filter = custom_item_templates.into_boxed();
    
    if let Some(inv_id) = inventory_id_param {
        filter = filter.filter(inventory_id.is_null().or(inventory_id.eq(inv_id)));
    } else {
        filter = filter.filter(inventory_id.is_null());
    }
    
    let templates = filter
        .load::<CustomItemTemplate>(&mut conn)
        .map_err(|e| {
            eprintln!("Error loading templates: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    
    Ok(HttpResponse::Ok().json(templates))
}

#[derive(serde::Deserialize)]
pub struct CreateTemplateRequest {
    pub inventory_id: Option<String>,
    pub name: String,
    pub default_unit: String,
}

#[actix_web::post("/api/inventory/templates")]
pub async fn create_custom_item_template(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<CreateTemplateRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let new_template = NewCustomItemTemplate {
        id: Uuid::new_v4().to_string(),
        inventory_id: req.inventory_id.clone(),
        name: req.name.clone(),
        default_unit: req.default_unit.clone(),
    };
    
    diesel::insert_into(crate::schema::custom_item_templates::table)
        .values(&new_template)
        .execute(&mut conn)
        .map_err(|e| {
            eprintln!("Error creating template: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error (likely duplicate name)")
        })?;
        
    Ok(HttpResponse::Created().json(new_template))
}

#[derive(serde::Deserialize)]
pub struct UpdateTemplateRequest {
    pub default_unit: String,
}

#[actix_web::put("/api/inventory/templates/{template_id}")]
pub async fn update_custom_item_template(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<UpdateTemplateRequest>,
) -> Result<HttpResponse> {
    let template_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::custom_item_templates::dsl::{custom_item_templates, default_unit};
    
    diesel::update(custom_item_templates.find(template_id_param))
        .set(default_unit.eq(&req.default_unit))
        .execute(&mut conn)
        .map_err(|e| {
            eprintln!("Error updating template: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
        
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::delete("/api/inventory/templates/{template_id}")]
pub async fn delete_custom_item_template(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let template_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    diesel::delete(crate::schema::custom_item_templates::table.find(template_id_param))
        .execute(&mut conn)
        .map_err(|e| {
            eprintln!("Error deleting template: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
        
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::get("/api/inventory/search")]
pub async fn search_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let search_query = query.get("q")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Query parameter 'q' required"))?;
    let inventory_id_param = query.get("inventory_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("inventory_id required"))?;
    
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::inventory_items::dsl::*;
    
    let pattern = format!("%{}%", search_query);
    
    let items = inventory_items
        .filter(inventory_id.eq(inventory_id_param))
        .filter(name.like(&pattern).or(barcode.like(&pattern)))
        .load::<InventoryItem>(&mut conn)
        .map_err(|e| {
            eprintln!("Error searching inventory: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    
    let results: Vec<ProductInfo> = items.into_iter().map(|item| {
        let product_info = item.product_data
            .and_then(|data| serde_json::from_str::<ProductInfo>(&data).ok());
            
        ProductInfo {
            id: Some(item.id),
            barcode: item.barcode,
            name: item.name,
            image_url: product_info.as_ref().and_then(|p| p.image_url.clone()),
            brand: product_info.as_ref().and_then(|p| p.brand.clone()),
            categories: product_info.map(|p| p.categories).unwrap_or_default(),
            unit: Some(item.unit),
        }
    }).collect();
    
    Ok(HttpResponse::Ok().json(results))
}

#[actix_web::get("/api/search")]
pub async fn search_product(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let search_query = query.get("q")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Query parameter 'q' required"))?;
    let inv_id = query.get("inventory_id");
    
    let mut conn = pool.get().expect("Failed to get DB connection");

    // 1. Search OpenFoodFacts
    let mut products = match openfoodfacts::search_products(search_query).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Search error: {:?}", e);
            vec![]
        }
    };

    // 2. If inventory_id provided, enrich products with existing config/unit
    if let Some(inventory_id_param) = inv_id {
        for product in products.iter_mut() {
            // Check existing items
            use crate::schema::inventory_items::dsl as ii;
            let existing = ii::inventory_items
                .filter(ii::inventory_id.eq(inventory_id_param))
                .filter(ii::name.eq(&product.name).or(ii::barcode.eq(&product.barcode)))
                .first::<InventoryItem>(&mut conn)
                .ok();
                
            if let Some(item) = existing {
                product.unit = Some(item.unit);
            } else {
                // Check templates
                use crate::schema::custom_item_templates::dsl as ct;
                let template = ct::custom_item_templates
                    .filter(ct::inventory_id.eq(inventory_id_param).or(ct::inventory_id.is_null()))
                    .filter(ct::name.eq(&product.name))
                    .order(ct::inventory_id.desc())
                    .first::<CustomItemTemplate>(&mut conn)
                    .ok();
                if let Some(t) = template {
                    product.unit = Some(t.default_unit);
                }
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(products))
}

#[actix_web::get("/api/product/{barcode}")]
pub async fn get_item_by_barcode(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let barcode_val = path.into_inner();
    let inv_id = query.get("inventory_id");
    
    let mut product = match openfoodfacts::get_product_by_barcode(&barcode_val).await {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Product not found"
        })))
    };

    if let Some(inventory_id_param) = inv_id {
        let mut conn = pool.get().expect("Failed to get DB connection");
        
        // Check existing items
        use crate::schema::inventory_items::dsl as ii;
        let existing = ii::inventory_items
            .filter(ii::inventory_id.eq(inventory_id_param))
            .filter(ii::barcode.eq(&barcode_val).or(ii::name.eq(&product.name)))
            .first::<InventoryItem>(&mut conn)
            .ok();
            
        if let Some(item) = existing {
            product.unit = Some(item.unit);
        } else {
            // Check templates
            use crate::schema::custom_item_templates::dsl as ct;
            let template = ct::custom_item_templates
                .filter(ct::inventory_id.eq(inventory_id_param).or(ct::inventory_id.is_null()))
                .filter(ct::name.eq(&product.name))
                .order(ct::inventory_id.desc())
                .first::<CustomItemTemplate>(&mut conn)
                .ok();
            if let Some(t) = template {
                product.unit = Some(t.default_unit);
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(product))
}

// --- User & Inventory Management Handlers ---

#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[actix_web::post("/api/users/register")]
pub async fn register_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let hashed_password = hash(&req.password, DEFAULT_COST).map_err(|e| {
        eprintln!("Hashing error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Internal server error")
    })?;
    
    use crate::schema::users::dsl::*;
    
    // Check if any users exist. If not, make first user an admin.
    let user_count = users.count().get_result::<i64>(&mut conn).unwrap_or(0);
    let role_val = if user_count == 0 { "admin" } else { "user" };
    
    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        email: req.email.clone(),
        password_hash: hashed_password,
        role: role_val.to_string(),
    };
    
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(&mut conn)
        .map_err(|e| {
            eprintln!("Error creating user: {:?}", e);
            actix_web::error::ErrorInternalServerError("Username or email likely taken")
        })?;
        
    Ok(HttpResponse::Created().json(serde_json::json!({"id": new_user.id, "username": new_user.username, "email": new_user.email, "role": new_user.role})))
}

#[derive(serde::Deserialize)]
pub struct CreateInventoryRequest {
    pub name: String,
    pub owner_id: String, // In real app, get this from auth token
}

#[actix_web::post("/api/inventories")]
pub async fn create_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<CreateInventoryRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let new_inv = NewInventory {
        id: Uuid::new_v4().to_string(),
        name: req.name.clone(),
        owner_id: req.owner_id.clone(),
    };
    
    diesel::insert_into(crate::schema::inventories::table)
        .values(&new_inv)
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    // Also add owner to inventory_users
    let inv_user = NewInventoryUser {
        inventory_id: new_inv.id.clone(),
        user_id: req.owner_id.clone(),
        role: "owner".to_string(),
    };
    
    diesel::insert_into(crate::schema::inventory_users::table)
        .values(&inv_user)
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Created().json(new_inv))
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[actix_web::post("/api/users/login")]
pub async fn login_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    let user = users
        .filter(username.eq(&req.username))
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid credentials"))?;
        
    if !verify(&req.password, &user.password_hash).unwrap_or(false) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid credentials"));
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({"id": user.id, "username": user.username, "email": user.email, "role": user.role})))
}

#[actix_web::post("/api/users/forgot-password")]
pub async fn forgot_password(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    let user = users
        .filter(email.eq(&req.email))
        .first::<User>(&mut conn)
        .ok();
        
    if let Some(user_val) = user {
        let token = Uuid::new_v4().to_string();
        let expiry = Utc::now().naive_utc() + Duration::hours(1);
        
        diesel::update(users.find(&user_val.id))
            .set((
                reset_token.eq(Some(&token)),
                reset_token_expiry.eq(Some(expiry)),
            ))
            .execute(&mut conn)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
            
        if let Err(e) = send_reset_email(&user_val.email, &token) {
            eprintln!("Email error: {}", e);
            // Don't return error to user for security (not leaking email existence)
        }
    }
    
    // Always return OK to avoid account enumeration
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "If that email exists in our system, a reset link has been sent."})))
}

#[actix_web::post("/api/users/reset-password")]
pub async fn reset_password(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    req: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    let user = users
        .filter(reset_token.eq(&req.token))
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid or expired token"))?;
        
    if let Some(expiry) = user.reset_token_expiry {
        if expiry < Utc::now().naive_utc() {
            return Err(actix_web::error::ErrorBadRequest("Token expired"));
        }
    } else {
        return Err(actix_web::error::ErrorBadRequest("Invalid token"));
    }
    
    let new_password_hash = hash(&req.new_password, DEFAULT_COST).map_err(|e| {
        eprintln!("Hashing error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Internal server error")
    })?;
    
    diesel::update(users.find(&user.id))
        .set((
            password_hash.eq(new_password_hash),
            reset_token.eq(None::<String>),
            reset_token_expiry.eq(None::<chrono::NaiveDateTime>),
        ))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Password updated successfully"})))
}

#[actix_web::put("/api/users/{user_id}")]
pub async fn update_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    let user_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    let mut update_count = 0;
    if let Some(ref new_username) = req.username {
        diesel::update(users.find(&user_id_param))
            .set(username.eq(new_username))
            .execute(&mut conn)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        update_count += 1;
    }
    
    if let Some(ref new_email) = req.email {
        diesel::update(users.find(&user_id_param))
            .set(email.eq(new_email))
            .execute(&mut conn)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        update_count += 1;
    }
    
    if update_count == 0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Nothing to update"})));
    }
    
    let user = users.find(&user_id_param)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorNotFound("User not found"))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({"id": user.id, "username": user.username, "email": user.email, "role": user.role})))
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[actix_web::get("/api/admin/users")]
pub async fn list_users(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let admin_id = query.get("admin_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("admin_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    // Check if requester is admin
    let requester = users.find(admin_id)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
        
    if requester.role != "admin" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }
    
    let all_users = users
        .load::<User>(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().json(all_users))
}

#[actix_web::put("/api/admin/users/{user_id}/role")]
pub async fn update_user_role(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<UpdateUserRoleRequest>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let target_user_id = path.into_inner();
    let admin_id = query.get("admin_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("admin_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    // Check if requester is admin
    let requester = users.find(admin_id)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
        
    if requester.role != "admin" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }
    
    diesel::update(users.find(&target_user_id))
        .set(role.eq(&req.role))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::put("/api/admin/users/{user_id}")]
pub async fn admin_update_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<AdminUpdateUserRequest>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let target_user_id = path.into_inner();
    let admin_id = query.get("admin_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("admin_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    // Check if requester is admin
    let requester = users.find(admin_id)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
        
    if requester.role != "admin" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }
    
    if let Some(ref new_username) = req.username {
        diesel::update(users.find(&target_user_id))
            .set(username.eq(new_username))
            .execute(&mut conn)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    }
    
    if let Some(ref new_email) = req.email {
        diesel::update(users.find(&target_user_id))
            .set(email.eq(new_email))
            .execute(&mut conn)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    }
    
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::post("/api/admin/users/{user_id}/reset-password")]
pub async fn admin_reset_password(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<AdminResetPasswordRequest>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let target_user_id = path.into_inner();
    let admin_id = query.get("admin_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("admin_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    // Check if requester is admin
    let requester = users.find(admin_id)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
        
    if requester.role != "admin" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }
    
    let hashed_password = hash(&req.new_password, DEFAULT_COST).map_err(|e| {
        eprintln!("Hashing error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Internal server error")
    })?;
    
    diesel::update(users.find(&target_user_id))
        .set(password_hash.eq(hashed_password))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::delete("/api/admin/users/{user_id}")]
pub async fn admin_delete_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let target_user_id = path.into_inner();
    let admin_id = query.get("admin_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("admin_id required"))?;
        
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    // Check if requester is admin
    let requester = users.find(admin_id)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
        
    if requester.role != "admin" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }
    
    diesel::delete(users.find(&target_user_id))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::post("/api/users/{user_id}/change-password")]
pub async fn change_password(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse> {
    let user_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::users::dsl::*;
    
    let user = users.find(&user_id_param)
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorNotFound("User not found"))?;
        
    if !verify(&req.current_password, &user.password_hash).unwrap_or(false) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid current password"));
    }
    
    let new_password_hash = hash(&req.new_password, DEFAULT_COST).map_err(|e| {
        eprintln!("Hashing error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Internal server error")
    })?;
    
    diesel::update(users.find(&user_id_param))
        .set(password_hash.eq(new_password_hash))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Password changed successfully"})))
}

#[actix_web::delete("/api/users/{user_id}")]
pub async fn delete_user(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    // In a real app, you'd want to handle dependent data (inventories, etc.)
    // For now, let's just delete the user. Diesel will fail if there are FK constraints not set to CASCADE.
    
    use crate::schema::users::dsl::*;
    
    diesel::delete(users.find(&user_id_param))
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to delete user: {}. You might need to delete their inventories first.", e)))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "User deleted successfully"})))
}

#[actix_web::get("/api/users/{user_id}/inventories")]
pub async fn get_user_inventories(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    // Join inventory_users with inventories to get the user's inventories
    let user_invs = crate::schema::inventory_users::table
        .inner_join(crate::schema::inventories::table)
        .filter(crate::schema::inventory_users::user_id.eq(user_id_param))
        .select(crate::schema::inventories::all_columns)
        .load::<Inventory>(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
    Ok(HttpResponse::Ok().json(user_invs))
}

#[derive(serde::Deserialize)]
pub struct ShareInventoryRequest {
    pub username: String,
    pub role: String,
}

#[actix_web::post("/api/inventories/{inventory_id}/share")]
pub async fn share_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<ShareInventoryRequest>,
) -> Result<HttpResponse> {
    let inventory_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");

    // 1. Find user by username
    use crate::schema::users::dsl::{users, username};
    let user_to_share_with = users
        .filter(username.eq(&req.username))
        .first::<User>(&mut conn)
        .map_err(|_| actix_web::error::ErrorNotFound("User not found"))?;

    // 2. Check if user is already in the inventory
    use crate::schema::inventory_users::dsl::{inventory_users, inventory_id, user_id};
    let existing_share = inventory_users
        .filter(inventory_id.eq(&inventory_id_param))
        .filter(user_id.eq(&user_to_share_with.id))
        .first::<InventoryUser>(&mut conn)
        .ok();

    if existing_share.is_some() {
        return Err(actix_web::error::ErrorConflict("User already has access to this inventory"));
    }

    // 3. Insert into inventory_users
    let new_inventory_user = NewInventoryUser {
        inventory_id: inventory_id_param,
        user_id: user_to_share_with.id,
        role: req.role.clone(),
    };

    diesel::insert_into(inventory_users)
        .values(&new_inventory_user)
        .execute(&mut conn)
        .map_err(|e| {
            eprintln!("Error sharing inventory: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to share inventory")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::get("/api/inventories/{inventory_id}/users")]
pub async fn get_inventory_users(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let inventory_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");

    use crate::schema::inventory_users::dsl as iu;
    use crate::schema::users::dsl as u;

    let results = iu::inventory_users
        .inner_join(u::users.on(iu::user_id.eq(u::id)))
        .filter(iu::inventory_id.eq(inventory_id_param))
        .select((u::id, u::username, u::email, iu::role))
        .load::<SharedUser>(&mut conn)
        .map_err(|e| {
            eprintln!("Error loading inventory users: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to load inventory users")
        })?;

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Deserialize)]
pub struct UnshareInventoryRequest {
    pub user_id: String,
}

#[actix_web::delete("/api/inventories/{inventory_id}/share")]
pub async fn unshare_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    path: web::Path<String>,
    req: web::Json<UnshareInventoryRequest>,
) -> Result<HttpResponse> {
    let inventory_id_param = path.into_inner();
    let mut conn = pool.get().expect("Failed to get DB connection");

    use crate::schema::inventory_users::dsl::{inventory_users, inventory_id, user_id};

    diesel::delete(
        inventory_users
            .filter(inventory_id.eq(inventory_id_param))
            .filter(user_id.eq(req.user_id.clone())),
    )
    .execute(&mut conn)
    .map_err(|e| {
        eprintln!("Error unsharing inventory: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to unshare inventory")
    })?;

    Ok(HttpResponse::Ok().finish())
}


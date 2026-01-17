use actix_web::{web, Result, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use crate::models::*;
use crate::schema::inventory_items;
use crate::openfoodfacts;
use uuid::Uuid;
use chrono::Utc;
use serde_json;

#[actix_web::get("/api/inventory")]
pub async fn show_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    let items = inventory_items::table
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
    
    let qty = req.quantity.unwrap_or(1);
    let prod_data = product_info.as_ref()
        .and_then(|_| serde_json::to_string(product_info.as_ref().unwrap()).ok());
    
    // Check if item with same barcode exists
    let existing_item = if let Some(ref barcode_val) = req.barcode {
        use crate::schema::inventory_items::dsl::barcode;
        inventory_items::table
            .filter(barcode.eq(barcode_val))
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else {
        None
    };
    
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
            barcode: req.barcode.clone(),
            name: item_name,
            quantity: qty,
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
        use crate::schema::inventory_items::dsl::barcode;
        inventory_items::table
            .filter(barcode.eq(barcode_val))
            .first::<InventoryItem>(&mut conn)
            .ok()
    } else {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Item ID or barcode required"
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
    
    let remove_quantity = req.quantity.unwrap_or(1);
    
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

#[actix_web::get("/api/inventory/search")]
pub async fn search_inventory(
    pool: web::Data<r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let search_query = query.get("q")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Query parameter 'q' required"))?;
    
    let mut conn = pool.get().expect("Failed to get DB connection");
    
    use crate::schema::inventory_items::dsl::*;
    
    let pattern = format!("%{}%", search_query);
    
    let items = inventory_items
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
            barcode: item.barcode,
            name: item.name,
            image_url: product_info.as_ref().and_then(|p| p.image_url.clone()),
            brand: product_info.as_ref().and_then(|p| p.brand.clone()),
            categories: product_info.map(|p| p.categories).unwrap_or_default(),
        }
    }).collect();
    
    Ok(HttpResponse::Ok().json(results))
}

#[actix_web::get("/api/search")]
pub async fn search_product(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let search_query = query.get("q")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Query parameter 'q' required"))?;
    
    match openfoodfacts::search_products(search_query).await {
        Ok(products) => Ok(HttpResponse::Ok().json(products)),
        Err(e) => {
            eprintln!("Search error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Search failed"
            })))
        }
    }
}

#[actix_web::get("/api/product/{barcode}")]
pub async fn get_item_by_barcode(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let barcode_val = path.into_inner();
    
    match openfoodfacts::get_product_by_barcode(&barcode_val).await {
        Ok(product) => Ok(HttpResponse::Ok().json(product)),
        Err(_) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Product not found"
        })))
    }
}

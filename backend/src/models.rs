use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::{inventory_items, users, inventories, inventory_users};

#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct InventoryItem {
    pub id: String,
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub product_data: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = inventory_items)]
pub struct NewInventoryItem {
    pub id: String,
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub product_data: Option<String>,
}

#[derive(Deserialize)]
pub struct AddItemRequest {
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Deserialize)]
pub struct RemoveItemRequest {
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub id: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct ProductInfo {
    pub barcode: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub brand: Option<String>,
    pub categories: Vec<String>,
}

// User Models
#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

// Inventory Models
#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = inventories)]
pub struct NewInventory {
    pub id: String,
    pub name: String,
    pub owner_id: String,
}

#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct InventoryUser {
    pub inventory_id: String,
    pub user_id: String,
    pub role: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = inventory_users)]
pub struct NewInventoryUser {
    pub inventory_id: String,
    pub user_id: String,
    pub role: String,
}

#[derive(Queryable, Serialize)]
pub struct SharedUser {
    pub id: String,
    pub username: String,
    pub role: String,
}

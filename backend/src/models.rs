use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::inventory_items;

#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct InventoryItem {
    pub id: String,
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
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub product_data: Option<String>,
}

#[derive(Deserialize)]
pub struct AddItemRequest {
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Deserialize)]
pub struct RemoveItemRequest {
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

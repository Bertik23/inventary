use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const DEFAULT_API_BASE: &str = "http://127.0.0.1:8080/api";

pub fn get_api_base() -> String {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.get_item("api_base").unwrap().unwrap_or_else(|| DEFAULT_API_BASE.to_string())
}

pub fn set_api_base(url: &str) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set_item("api_base", url).unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventoryItem {
    pub id: String,
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub product_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddItemRequest {
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveItemRequest {
    pub inventory_id: String,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Inventory {
    pub id: String,
    pub name: String,
    pub owner_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateInventoryRequest {
    pub name: String,
    pub owner_id: String,
}

pub async fn fetch_inventory(inventory_id: &str) -> Result<Vec<InventoryItem>, String> {
    let url = format!("{}/inventory?inventory_id={}", get_api_base(), inventory_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn add_item(req: AddItemRequest) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/add", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn remove_item(req: RemoveItemRequest) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/remove", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn search_products(query: &str) -> Result<Vec<ProductInfo>, String> {
    let url = format!("{}/search?q={}", get_api_base(), urlencoding::encode(query));
    fetch_json(&url, None::<&()>).await
}

pub async fn search_inventory_items(query: &str, inventory_id: &str) -> Result<Vec<ProductInfo>, String> {
    let url = format!("{}/inventory/search?q={}&inventory_id={}", get_api_base(), urlencoding::encode(query), inventory_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn get_product_by_barcode(barcode: &str) -> Result<ProductInfo, String> {
    let url = format!("{}/product/{}", get_api_base(), barcode);
    fetch_json(&url, None::<&()>).await
}

pub async fn login_user(req: AuthRequest) -> Result<User, String> {
    let url = format!("{}/users/login", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn register_user(req: AuthRequest) -> Result<User, String> {
    let url = format!("{}/users/register", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn get_user_inventories(user_id: &str) -> Result<Vec<Inventory>, String> {
    let url = format!("{}/users/{}/inventories", get_api_base(), user_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn create_inventory(req: CreateInventoryRequest) -> Result<Inventory, String> {
    let url = format!("{}/inventories", get_api_base());
    fetch_json(&url, Some(&req)).await
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SharedUser {
    pub id: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShareInventoryRequest {
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnshareInventoryRequest {
    pub user_id: String,
}

pub async fn get_inventory_users(inventory_id: &str) -> Result<Vec<SharedUser>, String> {
    let url = format!("{}/inventories/{}/users", get_api_base(), inventory_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn share_inventory(inventory_id: &str, req: ShareInventoryRequest) -> Result<(), String> {
    let url = format!("{}/inventories/{}/share", get_api_base(), inventory_id);
    fetch_json(&url, Some(&req)).await
}

pub async fn unshare_inventory(inventory_id: &str, req: UnshareInventoryRequest) -> Result<(), String> {
    let url = format!("{}/inventories/{}/share", get_api_base(), inventory_id);
    fetch_delete(&url, Some(&req)).await
}

async fn fetch_delete<T: Serialize>(
    url: &str,
    body: Option<&T>,
) -> Result<(), String> {
    let mut opts = RequestInit::new();
    opts.set_method("DELETE");
    opts.set_mode(RequestMode::Cors);
    
    if let Some(body_data) = body {
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
    
    Ok(())
}

async fn fetch_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    let opts = RequestInit::new();
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

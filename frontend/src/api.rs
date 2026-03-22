use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const DEFAULT_API_BASE: &str = "/api";

pub fn get_api_base() -> String {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    let base = local_storage
        .get_item("api_base")
        .unwrap()
        .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

    if base.starts_with("http") {
        base
    } else {
        // Construct full URL from current location if it's a relative path
        let location = window.location();
        let origin = location
            .origin()
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        format!("{}{}", origin, base)
    }
}

pub fn set_api_base(url: &str) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set_item("api_base", url).unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InventoryItem {
    pub id: String,
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub product_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddItemRequest {
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveItemRequest {
    pub inventory_id: String,
    pub barcode: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CustomItemTemplate {
    pub id: String,
    pub inventory_id: Option<String>,
    pub name: String,
    pub default_unit: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTemplateRequest {
    pub inventory_id: Option<String>,
    pub name: String,
    pub default_unit: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTemplateRequest {
    pub default_unit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProductInfo {
    pub id: Option<String>,
    pub barcode: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub brand: Option<String>,
    pub categories: Vec<String>,
    pub unit: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
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
    pub email: Option<String>,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateInventoryRequest {
    pub name: String,
    pub owner_id: String,
}

pub async fn fetch_inventory(
    inventory_id: &str,
) -> Result<Vec<InventoryItem>, String> {
    let url =
        format!("{}/inventory?inventory_id={}", get_api_base(), inventory_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn add_item(req: AddItemRequest) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/add", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn remove_item(
    req: RemoveItemRequest,
) -> Result<InventoryItem, String> {
    let url = format!("{}/inventory/remove", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn search_products(
    query: &str,
    inventory_id: Option<&str>,
) -> Result<Vec<ProductInfo>, String> {
    let url = if let Some(id) = inventory_id {
        format!(
            "{}/search?q={}&inventory_id={}",
            get_api_base(),
            urlencoding::encode(query),
            id
        )
    } else {
        format!("{}/search?q={}", get_api_base(), urlencoding::encode(query))
    };
    fetch_json(&url, None::<&()>).await
}

pub async fn search_inventory_items(
    query: &str,
    inventory_id: &str,
) -> Result<Vec<ProductInfo>, String> {
    let url = format!(
        "{}/inventory/search?q={}&inventory_id={}",
        get_api_base(),
        urlencoding::encode(query),
        inventory_id
    );
    fetch_json(&url, None::<&()>).await
}

pub async fn get_product_by_barcode(
    barcode: &str,
    inventory_id: Option<&str>,
) -> Result<ProductInfo, String> {
    let url = if let Some(id) = inventory_id {
        format!("{}/product/{}?inventory_id={}", get_api_base(), barcode, id)
    } else {
        format!("{}/product/{}", get_api_base(), barcode)
    };
    fetch_json(&url, None::<&()>).await
}

pub async fn get_custom_item_templates(
    inventory_id: Option<&str>,
) -> Result<Vec<CustomItemTemplate>, String> {
    let url = if let Some(id) = inventory_id {
        format!("{}/inventory/templates?inventory_id={}", get_api_base(), id)
    } else {
        format!("{}/inventory/templates", get_api_base())
    };
    fetch_json(&url, None::<&()>).await
}

pub async fn create_custom_item_template(
    req: CreateTemplateRequest,
) -> Result<CustomItemTemplate, String> {
    let url = format!("{}/inventory/templates", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn update_custom_item_template(
    template_id: &str,
    req: UpdateTemplateRequest,
) -> Result<(), String> {
    let url = format!("{}/inventory/templates/{}", get_api_base(), template_id);
    fetch_put(&url, Some(&req)).await
}

pub async fn delete_custom_item_template(
    template_id: &str,
) -> Result<(), String> {
    let url = format!("{}/inventory/templates/{}", get_api_base(), template_id);
    fetch_delete(&url, None::<&()>).await
}

pub async fn login_user(req: AuthRequest) -> Result<User, String> {
    let url = format!("{}/users/login", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn register_user(req: AuthRequest) -> Result<User, String> {
    let url = format!("{}/users/register", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn forgot_password(req: ForgotPasswordRequest) -> Result<(), String> {
    let url = format!("{}/users/forgot-password", get_api_base());
    let _: serde_json::Value = fetch_json(&url, Some(&req)).await?;
    Ok(())
}

pub async fn reset_password(req: ResetPasswordRequest) -> Result<(), String> {
    let url = format!("{}/users/reset-password", get_api_base());
    let _: serde_json::Value = fetch_json(&url, Some(&req)).await?;
    Ok(())
}

pub async fn update_user(
    user_id: &str,
    req: UpdateUserRequest,
) -> Result<User, String> {
    let url = format!("{}/users/{}", get_api_base(), user_id);
    fetch_put_json(&url, Some(&req)).await
}

pub async fn change_password(
    user_id: &str,
    req: ChangePasswordRequest,
) -> Result<(), String> {
    let url = format!("{}/users/{}/change-password", get_api_base(), user_id);
    fetch_json(&url, Some(&req)).await
}

pub async fn delete_user(user_id: &str) -> Result<(), String> {
    let url = format!("{}/users/{}", get_api_base(), user_id);
    fetch_delete(&url, None::<&()>).await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminUpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminResetPasswordRequest {
    pub new_password: String,
}

pub async fn list_users(admin_id: &str) -> Result<Vec<User>, String> {
    let url = format!("{}/admin/users?admin_id={}", get_api_base(), admin_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn update_user_role(
    admin_id: &str,
    user_id: &str,
    role: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/users/{}/role?admin_id={}",
        get_api_base(),
        user_id,
        admin_id
    );
    let req = UpdateUserRoleRequest {
        role: role.to_string(),
    };
    fetch_put(&url, Some(&req)).await
}

pub async fn admin_update_user(
    admin_id: &str,
    user_id: &str,
    req: AdminUpdateUserRequest,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/users/{}?admin_id={}",
        get_api_base(),
        user_id,
        admin_id
    );
    fetch_put(&url, Some(&req)).await
}

pub async fn admin_reset_password(
    admin_id: &str,
    user_id: &str,
    req: AdminResetPasswordRequest,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/users/{}/reset-password?admin_id={}",
        get_api_base(),
        user_id,
        admin_id
    );
    fetch_json(&url, Some(&req)).await
}

pub async fn admin_delete_user(
    admin_id: &str,
    user_id: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/users/{}?admin_id={}",
        get_api_base(),
        user_id,
        admin_id
    );
    fetch_delete(&url, None::<&()>).await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BufferProductRequest {
    pub barcode: String,
    pub name: String,
    pub brand: Option<String>,
    pub unit: Option<String>,
    pub added_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessProductRequest {
    pub action: String,
    pub name: String,
    pub brand: Option<String>,
    pub unit: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PendingProduct {
    pub barcode: String,
    pub name: String,
    pub brand: Option<String>,
    pub unit: Option<String>,
    pub added_by: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomProduct {
    pub barcode: String,
    pub name: String,
    pub brand: Option<String>,
    pub image_url: Option<String>,
    pub unit: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCustomProductRequest {
    pub name: String,
    pub brand: Option<String>,
    pub unit: Option<String>,
    pub action: Option<String>,
}

pub async fn buffer_unknown_product(
    req: BufferProductRequest,
) -> Result<(), String> {
    let url = format!("{}/products/buffer", get_api_base());
    fetch_json(&url, Some(&req)).await
}

pub async fn list_pending_products(
    admin_id: &str,
) -> Result<Vec<PendingProduct>, String> {
    let url = format!(
        "{}/admin/pending-products?admin_id={}",
        get_api_base(),
        admin_id
    );
    fetch_json(&url, None::<&()>).await
}

pub async fn process_pending_product(
    admin_id: &str,
    barcode: &str,
    req: ProcessProductRequest,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/products/{}/process?admin_id={}",
        get_api_base(),
        barcode,
        admin_id
    );
    fetch_json(&url, Some(&req)).await
}

pub async fn list_custom_products(
    admin_id: &str,
) -> Result<Vec<CustomProduct>, String> {
    let url = format!(
        "{}/admin/custom-products?admin_id={}",
        get_api_base(),
        admin_id
    );
    fetch_json(&url, None::<&()>).await
}

pub async fn update_custom_product(
    admin_id: &str,
    barcode: &str,
    req: UpdateCustomProductRequest,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/custom-products/{}?admin_id={}",
        get_api_base(),
        barcode,
        admin_id
    );
    fetch_put(&url, Some(&req)).await
}

pub async fn delete_custom_product(
    admin_id: &str,
    barcode: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/admin/custom-products/{}?admin_id={}",
        get_api_base(),
        barcode,
        admin_id
    );
    fetch_delete(&url, None::<&()>).await
}

pub async fn get_user_inventories(
    user_id: &str,
) -> Result<Vec<Inventory>, String> {
    let url = format!("{}/users/{}/inventories", get_api_base(), user_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn create_inventory(
    req: CreateInventoryRequest,
) -> Result<Inventory, String> {
    let url = format!("{}/inventories", get_api_base());
    fetch_json(&url, Some(&req)).await
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SharedUser {
    pub id: String,
    pub username: String,
    pub email: String,
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

pub async fn get_inventory_users(
    inventory_id: &str,
) -> Result<Vec<SharedUser>, String> {
    let url = format!("{}/inventories/{}/users", get_api_base(), inventory_id);
    fetch_json(&url, None::<&()>).await
}

pub async fn share_inventory(
    inventory_id: &str,
    req: ShareInventoryRequest,
) -> Result<(), String> {
    let url = format!("{}/inventories/{}/share", get_api_base(), inventory_id);
    fetch_json(&url, Some(&req)).await
}

pub async fn unshare_inventory(
    inventory_id: &str,
    req: UnshareInventoryRequest,
) -> Result<(), String> {
    let url = format!("{}/inventories/{}/share", get_api_base(), inventory_id);
    fetch_delete(&url, Some(&req)).await
}

async fn fetch_method<T: Serialize>(
    url: &str,
    method: &str,
    body: Option<&T>,
) -> Result<(), String> {
    let mut opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    if let Some(body_data) = body {
        let body_str =
            serde_json::to_string(body_data).map_err(|e| e.to_string())?;
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

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Response is not a Response: {:?}", e))?;

    if !resp.ok() {
        let status = resp.status();
        if let Ok(promise) = resp.json() {
            if let Ok(json) = JsFuture::from(promise).await {
                if let Ok(error_val) =
                    js_sys::Reflect::get(&json, &JsValue::from_str("error"))
                {
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

async fn fetch_delete<T: Serialize>(
    url: &str,
    body: Option<&T>,
) -> Result<(), String> {
    fetch_method(url, "DELETE", body).await
}

async fn fetch_put<T: Serialize>(
    url: &str,
    body: Option<&T>,
) -> Result<(), String> {
    fetch_method(url, "PUT", body).await
}

async fn fetch_put_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    fetch_json_with_method(url, "PUT", body).await
}

async fn fetch_json<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    let method = if body.is_some() { "POST" } else { "GET" };
    fetch_json_with_method(url, method, body).await
}

async fn fetch_json_with_method<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    method: &str,
    body: Option<&T>,
) -> Result<R, String> {
    let mut opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    if let Some(body_data) = body {
        let body_str =
            serde_json::to_string(body_data).map_err(|e| e.to_string())?;
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

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Response is not a Response: {:?}", e))?;

    if !resp.ok() {
        let status = resp.status();
        if let Ok(promise) = resp.json() {
            if let Ok(json) = JsFuture::from(promise).await {
                if let Ok(error_val) =
                    js_sys::Reflect::get(&json, &JsValue::from_str("error"))
                {
                    if let Some(msg) = error_val.as_string() {
                        return Err(msg);
                    }
                }
            }
        }
        return Err(format!("HTTP error: {}", status));
    }

    let json = JsFuture::from(
        resp.json()
            .map_err(|e| format!("Failed to get JSON: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Failed to deserialize: {:?}", e))
}

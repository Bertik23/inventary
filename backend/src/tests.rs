#[cfg(test)]
mod tests {
    use crate::handlers;
    use crate::models::*;
    use actix_web::{test, web, App};
    use diesel::prelude::*;
    use diesel::r2d2::{self, ConnectionManager};
    use diesel_migrations::{embed_migrations, MigrationHarness};
    use serde_json::json;

    pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
        embed_migrations!("migrations");

    fn init_test_pool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let db_id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_url = format!("file:memdb{}?mode=memory&cache=shared", db_id);
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);
        let pool = r2d2::Pool::builder()
            .max_size(5) // Enough for most handlers
            .build(manager)
            .expect("Failed to create pool");

        let mut conn = pool.get().expect("Failed to get DB connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        pool
    }

    async fn register_user<S, B>(
        app: &S,
        username: &str,
        email: &str,
        password: &str,
    ) -> serde_json::Value
    where
        S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        >,
        B: actix_web::body::MessageBody,
    {
        let req = test::TestRequest::post()
            .uri("/api/users/register")
            .set_json(&json!({
                "username": username,
                "email": email,
                "password": password
            }))
            .to_request();
        let resp = test::call_service(app, req).await;
        assert!(resp.status().is_success());
        test::read_body_json(resp).await
    }

    async fn login_user<S, B>(
        app: &S,
        username: &str,
        password: &str,
    ) -> serde_json::Value
    where
        S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        >,
        B: actix_web::body::MessageBody,
    {
        let req = test::TestRequest::post()
            .uri("/api/users/login")
            .set_json(&json!({
                "username": username,
                "password": password
            }))
            .to_request();
        let resp = test::call_service(app, req).await;
        assert!(resp.status().is_success());
        test::read_body_json(resp).await
    }

    #[actix_web::test]
    async fn test_full_user_flow() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::login_user)
                .service(handlers::update_user)
                .service(handlers::change_password),
        )
        .await;

        // 1. Register
        let user =
            register_user(&app, "testuser", "test@example.com", "pass123")
                .await;
        let user_id = user["id"].as_str().unwrap();

        // 2. Login
        let login_res = login_user(&app, "testuser", "pass123").await;
        assert_eq!(login_res["username"], "testuser");

        // 3. Update User
        let req = test::TestRequest::put()
            .uri(&format!("/api/users/{}", user_id))
            .set_json(&json!({
                "username": "newusername",
                "email": "new@example.com"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Change Password
        let req = test::TestRequest::post()
            .uri(&format!("/api/users/{}/change-password", user_id))
            .set_json(&json!({
                "current_password": "pass123",
                "new_password": "newpass123"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 5. Login with new password
        let login_res = login_user(&app, "newusername", "newpass123").await;
        assert_eq!(login_res["username"], "newusername");
    }

    #[actix_web::test]
    async fn test_inventory_and_items_flow() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::create_inventory)
                .service(handlers::add_item)
                .service(handlers::update_item)
                .service(handlers::remove_item)
                .service(handlers::show_inventory)
                .service(handlers::search_inventory),
        )
        .await;

        let user = register_user(&app, "owner", "owner@test.com", "pass").await;
        let user_id = user["id"].as_str().unwrap();

        // 1. Create Inventory
        let req = test::TestRequest::post()
            .uri("/api/inventories")
            .set_json(&json!({
                "name": "My Home",
                "owner_id": user_id
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let inv: NewInventory = test::read_body_json(resp).await;
        let inv_id = inv.id;

        // 2. Add Item
        let req = test::TestRequest::post()
            .uri("/api/inventory/add")
            .set_json(&json!({
                "inventory_id": inv_id,
                "name": "Milk",
                "quantity": 2.0,
                "unit": "liters"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let item: InventoryItemResponse = test::read_body_json(resp).await;
        let item_id = item.id;

        // 3. Update Item
        let req = test::TestRequest::put()
            .uri(&format!("/api/inventory/items/{}", item_id))
            .set_json(&json!({
                "quantity": 5.0
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Search Item
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/inventory/search?inventory_id={}&q=Mil",
                inv_id
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let search_results: Vec<ProductInfo> = test::read_body_json(resp).await;
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].name, "Milk");

        // 5. Remove Item (partial)
        let req = test::TestRequest::post()
            .uri("/api/inventory/remove")
            .set_json(&json!({
                "inventory_id": inv_id,
                "id": item_id,
                "quantity": 2.0
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let updated_item: InventoryItemResponse =
            test::read_body_json(resp).await;
        assert_eq!(updated_item.quantity, 3.0);

        // 6. Show Inventory
        let req = test::TestRequest::get()
            .uri(&format!("/api/inventory?inventory_id={}", inv_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let items: Vec<InventoryItemResponse> =
            test::read_body_json(resp).await;
        assert_eq!(items.len(), 1);
    }

    #[actix_web::test]
    async fn test_category_management() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::create_inventory)
                .service(handlers::create_inventory_category)
                .service(handlers::get_inventory_categories)
                .service(handlers::update_inventory_category)
                .service(handlers::delete_inventory_category),
        )
        .await;

        let user = register_user(&app, "user", "u@t.com", "p").await;
        let user_id = user["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri("/api/inventories")
            .set_json(&json!({"name": "Inv", "owner_id": user_id}))
            .to_request();
        let inv: NewInventory =
            test::read_body_json(test::call_service(&app, req).await).await;
        let inv_id = inv.id;

        // 1. Create Category
        let req = test::TestRequest::post()
            .uri(&format!("/api/inventories/{}/categories", inv_id))
            .set_json(&json!({"name": "Dairy"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let cat: NewInventoryCategory = test::read_body_json(resp).await;
        let cat_id = cat.id;

        // 2. Get Categories
        let req = test::TestRequest::get()
            .uri(&format!("/api/inventories/{}/categories", inv_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let cats: Vec<InventoryCategory> = test::read_body_json(resp).await;
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "Dairy");

        // 3. Update Category
        let req = test::TestRequest::put()
            .uri(&format!(
                "/api/inventories/{}/categories/{}",
                inv_id, cat_id
            ))
            .set_json(&json!({"name": "Milk & Cheese"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Delete Category
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/api/inventories/{}/categories/{}",
                inv_id, cat_id
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_admin_actions() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::list_users)
                .service(handlers::update_user_role)
                .service(handlers::admin_delete_user),
        )
        .await;

        // First user is admin (but wait, 'admin' is taken by migrations, and it makes our user regular)
        let admin =
            register_user(&app, "real_admin", "admin@t.com", "admin").await;
        let admin_id = admin["id"].as_str().unwrap();

        // MANUALLY PROMOTE TO ADMIN
        {
            let mut conn = pool.get().expect("Failed to get DB connection");
            use crate::schema::users::dsl::*;
            diesel::update(users.find(&admin_id))
                .set(role.eq("admin"))
                .execute(&mut conn)
                .expect("Failed to promote user to admin");
        }

        // Second user is regular user
        let user = register_user(&app, "real_user", "user@t.com", "user").await;
        let user_id = user["id"].as_str().unwrap();

        // 1. List Users
        let req = test::TestRequest::get()
            .uri(&format!("/api/admin/users?admin_id={}", admin_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let users: Vec<User> = test::read_body_json(resp).await;
        // There's a default user from migrations likely, or just our two.
        // Based on register_user handler, first user gets "admin", others "user".
        assert!(users.len() >= 2);

        // 2. Update Role
        let req = test::TestRequest::put()
            .uri(&format!(
                "/api/admin/users/{}/role?admin_id={}",
                user_id, admin_id
            ))
            .set_json(&json!({"role": "moderator"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 3. Delete User
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/api/admin/users/{}$?admin_id={}",
                user_id, admin_id
            ))
            .to_request();
        // Wait, I put a $ in URI by mistake in thought, but let's fix it.
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/api/admin/users/{}?admin_id={}",
                user_id, admin_id
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_sharing_flow() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::create_inventory)
                .service(handlers::share_inventory)
                .service(handlers::get_inventory_users)
                .service(handlers::unshare_inventory),
        )
        .await;

        let owner = register_user(&app, "owner", "o@t.com", "p").await;
        let other = register_user(&app, "other", "other@t.com", "p").await;
        let other_id = other["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri("/api/inventories")
            .set_json(&json!({"name": "Shared", "owner_id": owner["id"]}))
            .to_request();
        let inv: NewInventory =
            test::read_body_json(test::call_service(&app, req).await).await;
        let inv_id = inv.id;

        // 1. Share
        let req = test::TestRequest::post()
            .uri(&format!("/api/inventories/{}/share", inv_id))
            .set_json(&json!({"username": "other", "role": "editor"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 2. Get Shared Users
        let req = test::TestRequest::get()
            .uri(&format!("/api/inventories/{}/users", inv_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let users: Vec<SharedUser> = test::read_body_json(resp).await;
        assert_eq!(users.len(), 2); // owner and other

        // 3. Unshare
        let req = test::TestRequest::delete()
            .uri(&format!("/api/inventories/{}/share", inv_id))
            .set_json(&json!({"user_id": other_id}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_templates() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::create_custom_item_template)
                .service(handlers::get_custom_item_templates)
                .service(handlers::update_custom_item_template)
                .service(handlers::delete_custom_item_template),
        )
        .await;

        // 1. Create Template
        let req = test::TestRequest::post()
            .uri("/api/inventory/templates")
            .set_json(&json!({
                "name": "Bread",
                "default_unit": "pcs"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let t: NewCustomItemTemplate = test::read_body_json(resp).await;
        let t_id = t.id;

        // 2. Get Templates
        let req = test::TestRequest::get()
            .uri("/api/inventory/templates")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let templates: Vec<CustomItemTemplate> =
            test::read_body_json(resp).await;
        assert!(templates.iter().any(|x| x.name == "Bread"));

        // 3. Update Template
        let req = test::TestRequest::put()
            .uri(&format!("/api/inventory/templates/{}", t_id))
            .set_json(&json!({"default_unit": "kg"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Delete Template
        let req = test::TestRequest::delete()
            .uri(&format!("/api/inventory/templates/{}", t_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_pending_and_custom_products_flow() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user)
                .service(handlers::buffer_unknown_product)
                .service(handlers::list_pending_products)
                .service(handlers::process_pending_product)
                .service(handlers::list_custom_products)
                .service(handlers::get_item_by_barcode),
        )
        .await;

        let admin = register_user(&app, "admin_p", "ap@t.com", "p").await;
        let admin_id = admin["id"].as_str().unwrap();
        // Promote to admin
        {
            let mut conn = pool.get().expect("Failed to get DB connection");
            use crate::schema::users::dsl::*;
            diesel::update(users.find(&admin_id))
                .set(role.eq("admin"))
                .execute(&mut conn)
                .unwrap();
        }

        // 1. Buffer unknown product
        let req = test::TestRequest::post()
            .uri("/api/products/buffer")
            .set_json(&json!({
                "barcode": "non-existent-barcode-XYZ-123",
                "name": "Unknown Soda",
                "brand": "Generic",
                "unit": "can",
                "added_by": "user1"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 2. List pending
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/admin/pending-products?admin_id={}",
                admin_id
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let pending: Vec<PendingProduct> = test::read_body_json(resp).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "Unknown Soda");

        // 3. Process as local
        let req = test::TestRequest::post()
            .uri(&format!("/api/admin/products/non-existent-barcode-XYZ-123/process?admin_id={}", admin_id))
            .set_json(&json!({
                "action": "local",
                "name": "Generic Soda",
                "brand": "Generic",
                "unit": "0.33l"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Check custom products
        let req = test::TestRequest::get()
            .uri(&format!("/api/admin/custom-products?admin_id={}", admin_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let custom: Vec<CustomProduct> = test::read_body_json(resp).await;
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].name, "Generic Soda");

        // 5. Check get_item_by_barcode fallback
        let req = test::TestRequest::get()
            .uri("/api/product/non-existent-barcode-XYZ-123")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let product: ProductInfo = test::read_body_json(resp).await;
        assert_eq!(product.name, "Generic Soda");
        assert_eq!(product.unit, Some("0.33l".to_string()));
    }
}

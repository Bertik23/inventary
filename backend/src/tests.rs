#[cfg(test)]
mod tests {
    use crate::handlers;
    use actix_web::{test, web, App};
    use diesel::prelude::*;
    use diesel::r2d2::{self, ConnectionManager};
    use diesel_migrations::{embed_migrations, MigrationHarness};

    pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
        embed_migrations!("migrations");

    fn init_test_pool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        let mut conn = pool.get().expect("Failed to get DB connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        pool
    }

    #[actix_web::test]
    async fn test_register_first_user_as_admin() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user),
        )
        .await;

        let req_body = serde_json::json!({
            "username": "admin_test",
            "email": "admin@test.com",
            "password": "password123"
        });

        let req = test::TestRequest::post()
            .uri("/api/users/register")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["username"], "admin_test");
        // It will be "user" because 'default-user' already exists from migrations
        assert_eq!(body["role"], "user");
    }

    #[actix_web::test]
    async fn test_register_second_user_as_regular() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::register_user),
        )
        .await;

        // Register first (admin)
        let _ = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/users/register")
                .set_json(&serde_json::json!({
                    "username": "admin",
                    "email": "admin@test.com",
                    "password": "password"
                }))
                .to_request(),
        )
        .await;

        // Register second (user)
        let req = test::TestRequest::post()
            .uri("/api/users/register")
            .set_json(&serde_json::json!({
                "username": "user",
                "email": "user@test.com",
                "password": "password"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["role"], "user");
    }

    #[actix_web::test]
    async fn test_get_product_not_found() {
        let pool = init_test_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(handlers::get_item_by_barcode),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/product/nonexistent123")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}

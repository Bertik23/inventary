use actix_cors::Cors;
use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use std::env;

mod handlers;
mod models;
mod openfoodfacts;
mod schema;
#[cfg(test)]
mod tests;
// mod handlers_postgres; // Uncomment when Postgres handlers are implemented

pub type DbPool = r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>;
pub type DbPoolPostgres = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_type =
        env::var("DATABASE_TYPE").unwrap_or_else(|_| "sqlite".to_string());
    let _db_password = env::var("DATABASE_PASSWORD").ok();

    let static_files_dir = env::var("STATIC_FILES_DIR").ok();

    // Run migrations
    if database_type == "sqlite" {
        // Strip sqlite:// prefix if present (Diesel CLI uses it, but ConnectionManager needs just the path)
        let db_path = database_url
            .strip_prefix("sqlite://")
            .unwrap_or(&database_url);
        let manager =
            ConnectionManager::<diesel::SqliteConnection>::new(db_path);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        // Run migrations
        use diesel_migrations::{embed_migrations, MigrationHarness};
        pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
            embed_migrations!("migrations");
        let mut conn = pool.get().expect("Failed to get DB connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let bind_address = format!("{}:{}", host, port);

        let static_dir = static_files_dir.clone();

        println!("Server starting on http://{}", bind_address);
        HttpServer::new(move || {
            let cors = Cors::permissive();
            let mut app = App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(cors)
                .wrap(Logger::default())
                .service(handlers::add_item)
                .service(handlers::remove_item)
                .service(handlers::update_item)
                .service(handlers::show_inventory)
                .service(handlers::search_inventory)
                .service(handlers::search_product)
                .service(handlers::get_item_by_barcode)
                .service(handlers::register_user)
                .service(handlers::login_user)
                .service(handlers::forgot_password)
                .service(handlers::reset_password)
                .service(handlers::update_user)
                .service(handlers::change_password)
                .service(handlers::delete_user)
                .service(handlers::list_users)
                .service(handlers::update_user_role)
                .service(handlers::admin_update_user)
                .service(handlers::admin_reset_password)
                .service(handlers::admin_delete_user)
                .service(handlers::buffer_unknown_product)
                .service(handlers::list_pending_products)
                .service(handlers::process_pending_product)
                .service(handlers::list_custom_products)
                .service(handlers::update_custom_product)
                .service(handlers::delete_custom_product)
                .service(handlers::create_inventory)
                .service(handlers::get_user_inventories)
                .service(handlers::share_inventory)
                .service(handlers::get_inventory_users)
                .service(handlers::unshare_inventory)
                .service(handlers::get_inventory_categories)
                .service(handlers::create_inventory_category)
                .service(handlers::update_inventory_category)
                .service(handlers::delete_inventory_category)
                .service(handlers::get_custom_item_templates)
                .service(handlers::create_custom_item_template)
                .service(handlers::update_custom_item_template)
                .service(handlers::delete_custom_item_template);

            if let Some(ref dir) = static_dir {
                let dir_copy = dir.clone();
                app = app.service(
                    fs::Files::new("/", dir)
                        .index_file("index.html")
                        .default_handler(
                            move |req: actix_web::dev::ServiceRequest| {
                                let (http_req, _payload) = req.into_parts();
                                let dir_path = std::path::Path::new(&dir_copy);
                                let index_path = dir_path.join("index.html");
                                async move {
                                    let file = actix_files::NamedFile::open(
                                        index_path,
                                    )?;
                                    let res = file.into_response(&http_req);
                                    Ok(actix_web::dev::ServiceResponse::new(
                                        http_req, res,
                                    ))
                                }
                            },
                        )
                        .use_last_modified(true),
                );
            }

            app
        })
        .bind(bind_address)?
        .run()
        .await
    } else {
        // Note: Postgres support requires handlers that work with PgConnection
        // For now, we'll use SQLite handlers. To fully support Postgres,
        // you would need to create Postgres-specific handlers similar to handlers.rs
        // but using diesel::PgConnection.

        eprintln!("Postgres support is not fully implemented yet. Please use SQLite for now.");
        eprintln!("To add Postgres support, create handlers that use diesel::PgConnection.");
        std::process::exit(1);
    }
}

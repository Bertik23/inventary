use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use diesel::r2d2::{self, ConnectionManager};
use std::env;

mod models;
mod schema;
mod handlers;
mod openfoodfacts;
// mod handlers_postgres; // Uncomment when Postgres handlers are implemented

pub type DbPool = r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>;
pub type DbPoolPostgres = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let database_type = env::var("DATABASE_TYPE").unwrap_or_else(|_| "sqlite".to_string());
    let _db_password = env::var("DATABASE_PASSWORD").ok();

    // Run migrations
    if database_type == "sqlite" {
        // Strip sqlite:// prefix if present (Diesel CLI uses it, but ConnectionManager needs just the path)
        let db_path = database_url.strip_prefix("sqlite://").unwrap_or(&database_url);
        let manager = ConnectionManager::<diesel::SqliteConnection>::new(db_path);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        
        // Run migrations
        use diesel_migrations::{embed_migrations, MigrationHarness};
        pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get().expect("Failed to get DB connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        
        println!("Server starting on http://127.0.0.1:8080");
        HttpServer::new(move || {
            let cors = Cors::permissive();
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(cors)
                .service(handlers::add_item)
                .service(handlers::remove_item)
                .service(handlers::show_inventory)
                .service(handlers::search_inventory)
                .service(handlers::search_product)
                .service(handlers::get_item_by_barcode)
                .service(handlers::register_user)
                .service(handlers::login_user)
                .service(handlers::create_inventory)
                .service(handlers::get_user_inventories)
        })
        .bind("127.0.0.1:8080")?
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
        
        // Uncomment and implement when Postgres handlers are ready:
        /*
        let manager = ConnectionManager::<diesel::PgConnection>::new(&database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        
        diesel_migrations::embed_migrations!();
        embedded_migrations::run(&mut pool.get().unwrap())
            .expect("Failed to run migrations");
        
        println!("Server starting on http://127.0.0.1:8080");
        HttpServer::new(move || {
            let cors = Cors::permissive();
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(cors)
                .service(handlers_postgres::add_item)
                .service(handlers_postgres::remove_item)
                .service(handlers_postgres::show_inventory)
                .service(handlers::search_product)
                .service(handlers::get_item_by_barcode)
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
        */
    }
}

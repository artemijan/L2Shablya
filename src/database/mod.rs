pub mod user;

use crate::common::dto::config::Database;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

pub async fn new_db_pool(db_config: &Database) -> AnyPool {
    println!("Trying to create DB pool");
    AnyPoolOptions::new()
        .min_connections(db_config.min_connections as u32)
        .max_connections(db_config.max_connections as u32)
        .connect(&db_config.url)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to database at url {}. Error {}",
                &db_config.url, e
            )
        })
}

pub async fn run_migrations(pool: &AnyPool) {
    println!("Running migrations if exist.");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .unwrap_or_else(|e| panic!("Not able to apply DB migrations. {}", e));
}
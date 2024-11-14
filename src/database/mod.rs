pub mod user;

use crate::common::dto::config::Database;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;
pub type DBPool = AnyPool;

pub async fn new_db_pool(db_config: &Database) -> DBPool {
    println!("Trying to create DB pool");
    AnyPoolOptions::new()
        .min_connections(u32::from(db_config.min_connections))
        .max_connections(u32::from(db_config.max_connections))
        .connect(&db_config.url)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to database at url {}. Error {e}",
                &db_config.url
            )
        })
}

pub async fn run_migrations(pool: &DBPool) {
    println!("Running migrations if exist.");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .unwrap_or_else(|e| panic!("Not able to apply DB migrations. {e}"));
}

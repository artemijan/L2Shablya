pub mod chars;
pub mod user;

use crate::common::dto::Database;

#[cfg(feature = "postgres")]
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "postgres")]
pub type DBPool = sqlx::PgPool;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqlitePoolOptions;
#[cfg(feature = "sqlite")]
pub type DBPool = sqlx::SqlitePool;

pub async fn new_db_pool(db_config: &Database) -> DBPool {
    println!("Trying to create DB pool");
    #[cfg(feature = "postgres")]
    {
        PgPoolOptions::new()
            .min_connections(u32::from(db_config.min_connections))
            .max_connections(u32::from(db_config.max_connections))
            .connect(&db_config.url)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to connect to PostgreSQL database at url {}. Error {e}",
                    &db_config.url
                )
            })
    }

    #[cfg(feature = "sqlite")]
    {
        SqlitePoolOptions::new()
            .min_connections(u32::from(db_config.min_connections))
            .max_connections(u32::from(db_config.max_connections))
            .connect(&db_config.url)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to connect to SQLite database at url {}. Error {e}",
                    &db_config.url
                )
            })
    }
}

pub async fn run_migrations(pool: &DBPool) {
    println!("Running migrations if exist.");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .unwrap_or_else(|e| panic!("Not able to apply DB migrations. {e}"));
}

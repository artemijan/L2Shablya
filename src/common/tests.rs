use crate::database::run_migrations;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;
#[cfg(feature = "postgres")]
pub type DBPool = sqlx::PgPool;
#[cfg(feature = "postgres")]
static DB_POOL: OnceCell<SqlitePool> = OnceCell::const_new();

#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
#[cfg(feature = "sqlite")]
pub type DBPool = SqlitePool;
#[cfg(feature = "sqlite")]
static DB_POOL: OnceCell<SqlitePool> = OnceCell::const_new();

async fn setup_test_db() -> DBPool {
    #[cfg(feature = "sqlite")]
    {
        let database_url = "sqlite::memory:";
        sqlx::any::install_default_drivers();
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .expect("Failed to create pool");
        run_migrations(&pool).await;
        pool
    }
    #[cfg(feature = "postgres")]
    {
        let database_url = "sqlite::memory:"; //todo implement setup for tests
        sqlx::any::install_default_drivers();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .expect("Failed to create pool");
        run_migrations(&pool).await;
        pool
    }
}

pub async fn get_test_db() -> DBPool {
    DB_POOL.get_or_init(setup_test_db).await.clone()
}

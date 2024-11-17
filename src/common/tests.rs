use crate::database::run_migrations;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<AnyPool> = OnceCell::const_new();

async fn setup_test_db() -> AnyPool {
    let database_url = "sqlite::memory:";
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to create pool");
    run_migrations(&pool).await;
    pool
}

pub async fn get_test_db() -> AnyPool {
    DB_POOL
        .get_or_init(setup_test_db)
        .await.clone()
}

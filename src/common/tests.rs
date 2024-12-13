use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx;
use tokio::sync::OnceCell;

pub type DBPool = DatabaseConnection;

static DB_POOL: OnceCell<DBPool> = OnceCell::const_new();

async fn setup_test_db() -> DBPool {
    let opt = ConnectOptions::new("sqlite::memory:");

    let conn = Database::connect(opt)
        .await
        .expect("Failed to connect to the database");
    Migrator::up(&conn, None)
        .await
        .expect("Failed to migrate the database");
    conn
}

pub async fn get_test_db() -> DBPool {
    DB_POOL.get_or_init(setup_test_db).await.clone()
}

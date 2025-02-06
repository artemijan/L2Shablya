use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub type DBPool = DatabaseConnection;

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
    setup_test_db().await
}

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

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn test_hex_id() -> Vec<u8> {
    i128::from_str_radix("-2ad66b3f483c22be097019f55c8abdf0", 16)
        .unwrap()
        .to_be_bytes()
        .to_vec()
}

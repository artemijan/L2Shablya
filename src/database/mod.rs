use crate::common::dto::Database;
use sea_orm::{ConnectOptions, Database as DB, DatabaseConnection};
use std::time::Duration;
use tracing::instrument;
pub type DBPool = DatabaseConnection;

#[instrument]
pub async fn new_db_pool(db_config: &Database) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(&db_config.url);
    opt.max_connections(u32::from(db_config.max_connections))
        .min_connections(u32::from(db_config.min_connections))
        .connect_timeout(Duration::from_secs(db_config.connect_timeout))
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .sqlx_logging(true);

    DB::connect(opt)
        .await
        .expect("Failed to connect to the database")
}

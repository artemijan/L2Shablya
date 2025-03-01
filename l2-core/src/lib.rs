use crate::dto::Database as DBConfig;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tokio::task::spawn_blocking;
use tracing::instrument;

pub mod config;
pub mod constants;
pub mod crypt;
pub mod dto;
pub mod enums;
pub mod errors;
pub mod message_broker;
pub mod network;
pub mod session;
pub mod shared_packets;
pub mod str;
pub mod traits;
pub mod model;
pub mod bitmask;

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn hash_password(password: &str) -> anyhow::Result<String> {
    let pwd = password.to_owned();
    spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        // code here to hash the password
        // might take a second of CPU time
        argon2
            .hash_password(pwd.as_bytes(), &salt)
            .expect("Unable to hash password")
            .to_string()
    })
    .await
    .map_err(Into::into)
}

#[instrument]
pub async fn new_db_pool(db_config: &DBConfig) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(&db_config.url);
    opt.max_connections(u32::from(db_config.max_connections))
        .min_connections(u32::from(db_config.min_connections))
        .connect_timeout(Duration::from_secs(db_config.connect_timeout))
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .sqlx_logging(false);

    Database::connect(opt)
        .await
        .expect("Failed to connect to the database")
}

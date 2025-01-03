use std::time::Duration;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::task::spawn_blocking;
use tracing::instrument;
use crate::dto::Database as DBConfig;

pub mod constants;
pub mod dto;
pub mod errors;
pub mod network;
pub mod packets;
pub mod session;
pub mod str;
pub mod tests;
pub mod traits;
pub mod config;
pub mod crypt;
pub mod message_broker;

pub async fn hash_password(password: &str) -> anyhow::Result<String> {
    let pwd = password.to_owned();
    spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        // code here to hash the password
        // might take a second of CPU time
        argon2
            .hash_password(pwd.as_bytes(), &salt)
            .unwrap()
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
        .sqlx_logging(true);

    Database::connect(opt)
        .await
        .expect("Failed to connect to the database")
}
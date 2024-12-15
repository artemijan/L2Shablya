use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;
use tokio::task::spawn_blocking;

pub mod constants;
pub mod dto;
pub mod errors;
pub mod network;
pub mod packets;
pub mod session;
pub mod str;
#[cfg(test)]
pub mod tests;
pub mod traits;
pub mod config;

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

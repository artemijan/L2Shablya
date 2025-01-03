use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::entity::prelude::*;
use tokio::task::spawn_blocking;
use tracing::error;
use crate::entities::user::{Column, Entity, Model};

impl Model {
    pub async fn verify_password(&self, password: &str) -> bool {
        let pwd = password.to_owned();
        let pwd_hash = self.password.clone();
        let res = spawn_blocking(move || {
            let Ok(parsed_hash) = PasswordHash::new(&pwd_hash) else {
                error!("Can not generate a hash for password");
                return false;
            };
            Argon2::default()
                .verify_password(pwd.as_bytes(), &parsed_hash)
                .is_ok()
        })
            .await;
        res.unwrap_or_else(|err| {
            error!("Failed to spawn blocking thread to generate hash: {err}");
            false
        })
    }
    ///
    ///
    /// # Arguments
    ///
    /// * `db_pool`:
    /// * `username`:
    ///
    /// returns: Result<Option<Model>, Error>
    /// # Errors
    pub async fn find_some_by_username(
        db_pool: &DatabaseConnection,
        username: &str,
    ) -> anyhow::Result<Option<Model>> {
        Ok(Entity::find()
            .filter(Column::Username.contains(username))
            .one(db_pool)
            .await?)
    }
    ///
    ///
    /// # Arguments
    ///
    /// * `db_pool`:
    /// * `username`:
    ///
    /// returns: Result<Option<Model>, Error>
    /// # Errors
    pub async fn find_by_username(
        db_pool: &DatabaseConnection,
        username: &str,
    ) -> anyhow::Result<Model> {
        Entity::find()
            .filter(Column::Username.contains(username))
            .one(db_pool)
            .await?
            .ok_or_else(|| anyhow!("User not found {username}"))
    }
}
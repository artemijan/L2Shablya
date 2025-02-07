use crate::entities::user::{Column, Entity, Model};
use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::entity::prelude::*;
use tokio::task::spawn_blocking;
use tracing::error;

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

#[cfg(test)]
mod tests {
    use crate::dao::user;
    use crate::test_factories::factories::user_factory;
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_verify_password_works() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |mut u|{
            u.username = "test".to_owned();
            u.password = "$argon2id$v=19$m=19456,t=2,p=1$OnSjOZTt6Or9MxtqrcrGhw$GAY7oGKMMAQbd6tvWB96IjA6yxvZy2PMD2MEpHbmWS0".to_owned();
            u
        }).await;
        let is_ok1 = user.verify_password("test").await;
        let is_ok2 = user.verify_password("admin").await;
        assert!(!is_ok1);
        assert!(is_ok2);
    }
    #[tokio::test]
    async fn test_find_some_by_username() {
        let db_pool = get_test_db().await;
        let _ = user_factory(&db_pool, |mut u| {
            u.username = "test".to_owned();
            u.password = "pwd".to_owned();
            u
        })
        .await;
        let user1 = user::Model::find_some_by_username(&db_pool, "admin")
            .await
            .unwrap();
        let user2 = user::Model::find_some_by_username(&db_pool, "test")
            .await
            .unwrap();
        assert!(user1.is_none());
        assert!(user2.is_some());
    } 
    #[tokio::test]
    async fn test_find_by_username() {
        let db_pool = get_test_db().await;
        let the_user = user_factory(&db_pool, |mut u| {
            u.username = "test".to_owned();
            u.password = "pwd".to_owned();
            u
        })
        .await;
        let user = user::Model::find_by_username(&db_pool, "test")
            .await
            .unwrap();
        assert_eq!(user.username, the_user.username);
    }
}

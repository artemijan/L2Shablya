use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use sqlx::{AnyPool, Error, FromRow};
use tokio::task::spawn_blocking;

/// This is a struct which is simply a DTO to get/store data in DB
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
}

#[allow(unused)]
impl User {
    pub async fn fetch_by_username(
        db_pool: &AnyPool,
        username: &str,
    ) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as(
            "select id,username,password from user where username=$1",
        ).bind(username)
            .fetch_optional(db_pool)
            .await?;
        Ok(user)
    }

    /// cpu bound need to spawn a separate thread
    pub async fn hash_password(password: &str) -> String {
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
            .unwrap()
    }
    /// cpu bound need to spawn a separate thread
    pub async fn verify_password(&self, password: &str) -> bool {
        let pwd = password.to_owned();
        let pwd_hash = self.password.to_owned();
        spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&pwd_hash).unwrap();
            Argon2::default()
                .verify_password(pwd.as_bytes(), &parsed_hash)
                .is_ok()
        })
            .await
            .unwrap()
    }
}

use crate::database::DBPool;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use sqlx::{query_as, Error, FromRow};
use tokio::task::spawn_blocking;

/// This is a struct which is simply a DTO to get/store data in DB
#[derive(Debug, Clone, FromRow, Default)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub ban_duration: Option<i64>,
    pub ban_ip: Option<String>,
    pub password: String,
    pub access_level: i64,
}

#[allow(unused)]
impl User {
    pub async fn fetch_by_username(
        db_pool: &DBPool,
        username: &str,
    ) -> Result<Option<Self>, Error> {
        let user =
            sqlx::query_as("select id,username,password,access_level from user where username=$1")
                .bind(username)
                .fetch_optional(db_pool)
                .await?;
        Ok(user)
    }

    /// cpu bound need to spawn a separate thread
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
    /// cpu bound need to spawn a separate thread
    pub async fn verify_password(&self, password: &str) -> bool {
        let pwd = password.to_owned();
        let pwd_hash = self.password.clone();
        spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&pwd_hash).unwrap();
            Argon2::default()
                .verify_password(pwd.as_bytes(), &parsed_hash)
                .is_ok()
        })
        .await
        .unwrap()
    }
    pub async fn change_access_level(
        db_pool: &DBPool,
        access_level: i32,
        username: &str,
    ) -> anyhow::Result<User> {
        let user = sqlx::query_as("UPDATE user SET access_level = $1 where username=$2")
            .bind(access_level)
            .bind(username)
            .fetch_one(db_pool)
            .await?;
        Ok(user)
    }
    pub async fn req_temp_ban(
        db_pool: &DBPool,
        username: &str,
        ban_duration: i64,
        ip: &str,
    ) -> anyhow::Result<User> {
        let user = query_as!(
            User,
            r#"
            UPDATE user SET 
                ban_duration = ?, 
                ban_ip = ? 
            WHERE username = ? 
            RETURNING 
                id, 
                username, 
                password, access_level, ban_ip, ban_duration
            "#,
            ban_duration,
            ip,
            username
        )
        .fetch_one(db_pool)
        .await?;
        Ok(user)
    }
    pub async fn new(db_pool: &DBPool, username: &str, password: &str) -> anyhow::Result<User> {
        let password_hash = Self::hash_password(password).await?;
        let user = query_as!(
            User,
            r#"
            INSERT INTO user (username, password, access_level, ban_ip, ban_duration) 
            VALUES (?, ?, ?, NULL, NULL)
            RETURNING id, username, password, access_level, ban_ip, ban_duration
            "#,
            username,
            password_hash,
            0,
        )
        .fetch_one(db_pool)
        .await?;
        Ok(user)
    }
}

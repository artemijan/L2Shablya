use crate::database::DBPool;
use chrono::NaiveDateTime;
use sqlx::{query_as, Error, FromRow, Row};

/// This is a struct which is simply a DTO to get/store data in DB
#[derive(Debug, Clone, FromRow, Default)]
pub struct Character {
    pub id: Option<i64>,
    pub name: String,
    pub level: i64,
    pub delete_at: Option<NaiveDateTime>,
    pub user_id: i64, // Foreign key referencing the `users` table
}

#[allow(unused)]
impl Character {
    pub async fn fetch_by_account(db_pool: &DBPool, account: &str) -> Result<Vec<Self>, Error> {
        let chars: Vec<Character> = query_as!(
            Character,
            r#"
            SELECT 
                character.id,
                character.name,
                character.level,
                character.user_id,
                character.delete_at
            FROM 
                character
            INNER JOIN 
                user 
            ON 
                character.user_id = user.id
            WHERE 
                user.username = ?
            "#,
            account
        )
        .fetch_all(db_pool)
        .await?;

        Ok(chars)
    }
}

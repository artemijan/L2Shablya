use sea_orm::{DatabaseConnection, DbErr, JoinType, QuerySelect};
use crate::entities::character::{Entity, Model};
use crate::entities::user;
use sea_orm::entity::prelude::*;

impl Model {

    /// # Errors
    /// - `DbErr`
    /// - `QueryError`
    ///
    pub async fn find_characters_by_username(
        db_pool: &DatabaseConnection,
        username: &str,
    ) -> Result<Vec<Model>, DbErr> {
        let characters = Entity::find()
            .join(JoinType::InnerJoin, Entity::has_many(user::Entity).into())
            .filter(user::Column::Username.eq(username))
            .all(db_pool)
            .await?;
        Ok(characters)
    }
}
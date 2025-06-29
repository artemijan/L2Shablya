use crate::entities::skill;
use crate::DBPool;
use sea_orm::ColumnTrait;
use sea_orm::{DbErr, EntityTrait, QueryFilter};

impl skill::Model {
    ///
    /// # Errors
    /// - when DB connection is lost
    pub async fn char_skills(db_pool: &DBPool, id: i32) -> Result<Vec<skill::Model>, DbErr> {
        skill::Entity::find()
            .filter(skill::Column::CharId.eq(id))
            .all(db_pool)
            .await
    }
}

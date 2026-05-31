use crate::entities::skill;
use crate::DBPool;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveValue, DbErr, EntityTrait, QueryFilter};

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

    ///
    /// # Errors
    /// - when DB connection is lost
    pub async fn insert_skills(db_pool: &DBPool, skills: Vec<skill::Model>) -> Result<(), DbErr> {
        if skills.is_empty() {
            return Ok(());
        }
        let active_models: Vec<skill::ActiveModel> = skills
            .into_iter()
            .map(|s| skill::ActiveModel {
                id: ActiveValue::Set(s.id),
                char_id: ActiveValue::Set(s.char_id),
                level: ActiveValue::Set(s.level),
                sub_level: ActiveValue::Set(s.sub_level),
                class_index: ActiveValue::Set(s.class_index),
            })
            .collect();

        skill::Entity::insert_many(active_models)
            .exec(db_pool)
            .await?;
        Ok(())
    }
}

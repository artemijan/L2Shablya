use crate::dao::item::LocType;
use crate::entities::{character, item, user};
use crate::DBPool;
use chrono::{Duration, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DbErr, JoinType, Order, QueryOrder, QuerySelect};

#[allow(clippy::missing_errors_doc)]
impl character::Model {
    pub async fn char_exists(db_pool: &DBPool, name: &str) -> Result<bool, DbErr> {
        Ok(character::Entity::find()
            .filter(character::Column::Name.eq(name))
            .one(db_pool)
            .await?
            .is_some())
    }
    pub async fn create_char(
        db_pool: &DBPool,
        c: character::Model,
    ) -> Result<character::Model, DbErr> {
        let mut active_model: character::ActiveModel = c.into();
        active_model.id = ActiveValue::NotSet;
        active_model.last_access = ActiveValue::Set(Some(Utc::now().into()));
        active_model.insert(db_pool).await
    }
    pub async fn delete_char(
        db_pool: &DBPool,
        c: character::Model,
    ) -> Result<character::Model, DbErr> {
        let mut active_model: character::ActiveModel = c.into();
        active_model.delete_at = ActiveValue::Set(Some((Utc::now() + Duration::days(7)).into()));
        active_model.update(db_pool).await
    }
    pub async fn restore_char(
        db_pool: &DBPool,
        c: character::Model,
    ) -> Result<character::Model, DbErr> {
        let mut active_model: character::ActiveModel = c.into();
        active_model.delete_at = ActiveValue::Set(None);
        active_model.update(db_pool).await
    }
    pub async fn find_by_username(
        db_pool: &DBPool,
        username: &str,
    ) -> Result<Vec<character::Model>, DbErr> {
        let characters = character::Entity::find()
            .join(
                JoinType::InnerJoin,
                character::Entity::has_many(user::Entity).into(),
            )
            .filter(user::Column::Username.eq(username))
            .all(db_pool)
            .await?;
        Ok(characters)
    }

    pub async fn get_with_items_and_vars(
        db_pool: &DBPool,
        username: &str,
        loc_type: LocType,
    ) -> anyhow::Result<Vec<(character::Model, Vec<item::Model>)>> {
        let characters = character::Entity::find()
            .order_by(character::Column::DeleteAt, Order::Asc)
            .order_by(character::Column::CreatedAt, Order::Asc)
            .join(JoinType::InnerJoin, character::Relation::User.def())
            .filter(user::Column::Username.eq(username))
            .find_with_related(item::Entity)
            .all(db_pool)
            .await?;

        Ok(characters
            .into_iter()
            .map(|(char, items)| {
                (
                    char,
                    //todo optimize query maybe there is a way to do filtering on DB level for items
                    items.into_iter().filter(|i| i.loc == loc_type).collect(),
                )
            })
            .collect())
    }

    #[must_use]
    pub fn get_lvl(&self) -> u8 {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_factories::factories::{char_factory, item_factory, user_factory};
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_get_items_and_vars() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut c| {
            c.user_id = user.id;
            c
        })
        .await;
        let item = item_factory(&db_pool, |mut c| {
            c.item_id = 10;
            c.owner = char.id;
            c.loc_data = 5; // RHand
            c
        })
        .await;

        let chars =
            character::Model::get_with_items_and_vars(&db_pool, "admin", LocType::Paperdoll)
                .await
                .unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].0.id, char.id);
        assert_eq!(chars[0].1.len(), 1);
        assert_eq!(chars[0].1[0].loc, LocType::Paperdoll);
        assert_eq!(chars[0].1[0].item_id, item.item_id);
    }
    #[tokio::test]
    async fn test_find_by_username() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut c| {
            c.user_id = user.id;
            c
        })
        .await;

        let chars = character::Model::find_by_username(&db_pool, "admin")
            .await
            .unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].user_id, user.id);
        assert_eq!(chars[0].id, char.id);
    }

    #[tokio::test]
    async fn test_delete_char() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut c| {
            c.user_id = user.id;
            c
        })
        .await;
        let deleted_char = character::Model::delete_char(&db_pool, char).await.unwrap();
        let expected_delete_at = Utc::now() + Duration::days(7);
        let actual_delete_at = deleted_char.delete_at.unwrap().with_timezone(&Utc);
        //char should be deleted in 7 days
        assert_eq!((actual_delete_at - expected_delete_at).num_hours(), 0);
    }
    #[tokio::test]
    async fn test_restore_char() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut c| {
            c.user_id = user.id;
            c.delete_at = Some((Utc::now() + Duration::days(3)).into());
            c
        })
        .await;
        let restored_char = character::Model::restore_char(&db_pool, char)
            .await
            .unwrap();
        assert!(restored_char.delete_at.is_none());
    }
    #[tokio::test]
    async fn test_char_exists() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let _ = char_factory(&db_pool, |mut c| {
            c.user_id = user.id;
            c
        })
        .await;
        let exists = character::Model::char_exists(&db_pool, "Admin")
            .await
            .unwrap();
        assert!(exists);
    }
    #[tokio::test]
    async fn test_create_char() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = character::Model {
            name: "Tester".to_string(),
            level: 0,
            user_id: user.id,
            is_female: false,
            ..Default::default()
        };
        let new_char = character::Model::create_char(&db_pool, char).await.unwrap();
        assert_eq!(new_char.name, "Tester");
        assert!(new_char.last_access.is_some());
    }
}

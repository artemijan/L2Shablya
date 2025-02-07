use crate::dao::char_info::CharacterInfo;
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
    ) -> Result<Vec<CharacterInfo>, DbErr> {
        let characters = character::Entity::find()
            .order_by(character::Column::DeleteAt, Order::Asc)
            .order_by(character::Column::CreatedAt, Order::Asc)
            .join(JoinType::InnerJoin, character::Relation::User.def())
            .filter(user::Column::Username.eq(username))
            .find_with_related(item::Entity)
            .all(db_pool)
            .await?;

        characters
            .into_iter()
            .map(|(char, items)| {
                CharacterInfo::new(
                    char,
                    //todo optimize query maybe there is a way to do filtering on DB level for items
                    items.into_iter().filter(|i| i.loc == loc_type).collect(),
                )
            })
            .collect()
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn get_lvl(&self) -> u8 {
        self.level as u8
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn restore_visible_inventory(
        &self,
        items: &Vec<item::Model>,
    ) -> Result<[[i32; 4]; 33], DbErr> {
        let mut result = [[0; 4]; 33];
        for item in items {
            if item.loc == LocType::Paperdoll {
                let slot = item.loc_data;
                result[slot as usize][0] = item.id;
                result[slot as usize][1] = item.item_id;
                result[slot as usize][2] = item.enchant_level;
                // todo: result[slot as usize][3] = vars
                //     .get(item_variable::Model::VISUAL_ID)
                //     .unwrap_or(&"0".to_string())
                //     .parse()
                //     .unwrap_or(0);
                if result[slot as usize][3] > 0 {
                    // fix for hair appearance conflicting with original model
                    result[slot as usize][1] = result[slot as usize][3];
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::char_info::PaperDoll;
    use crate::test_factories::factories::{char_factory, item_factory, user_factory};
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_works() {
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
            c.loc_data = PaperDoll::RHand as i32;
            c
        })
        .await;

        let chars =
            character::Model::get_with_items_and_vars(&db_pool, "admin", LocType::Paperdoll)
                .await
                .unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char_model.id, char.id);
        assert_eq!(chars[0].items.len(), 1);
        assert_eq!(chars[0].items[0].loc, LocType::Paperdoll);
        assert_eq!(chars[0].items[0].item_id, item.item_id);
    }
}

use crate::dao::char_info::CharacterInfo;
use crate::dao::item::LocType;
use crate::entities::{character, item, user};
use crate::DBPool;
use sea_orm::entity::prelude::*;
use sea_orm::{DbErr, JoinType, Order, QueryOrder, QuerySelect};

#[allow(clippy::missing_errors_doc)]
impl character::Model {
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
            .order_by(character::Column::CreatedAt, Order::Asc)
            .join(
                JoinType::InnerJoin,
                character::Entity::has_many(user::Entity).into(),
            )
            .filter(user::Column::Username.eq(username))
            .find_with_related(item::Entity)
            .filter(item::Column::Loc.eq(loc_type))
            .all(db_pool)
            .await?;

        characters
            .into_iter()
            .map(|(char, items)| CharacterInfo::new(char, items))
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
                // result[slot as usize][3] = vars
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

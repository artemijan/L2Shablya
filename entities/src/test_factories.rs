#[cfg(feature = "test-utils")]
pub mod factories {
    use crate::dao::char_info::{PaperDoll, Race};
    use crate::dao::item::LocType;
    use crate::entities;
    use crate::entities::{character, user};
    use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, TryIntoModel};
    use test_utils::utils::DBPool;

    #[allow(clippy::missing_panics_doc)]
    pub async fn user_factory<F>(db_pool: &DBPool, create_fn: F) -> user::Model
    where
        F: FnOnce(user::Model) -> user::Model,
    {
        let model = user::Model {
            username: "admin".to_string(),
            access_level: 0,
            password: "hashed_pass".to_string(),
            ..Default::default()
        };
        let mut active_model = create_fn(model).into_active_model();
        active_model.id = ActiveValue::NotSet;
        active_model
            .into_active_model()
            .insert(db_pool)
            .await
            .unwrap()
            .try_into_model()
            .unwrap()
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn char_factory<F>(db_pool: &DBPool, create_fn: F) -> character::Model
    where
        F: FnOnce(character::Model) -> character::Model,
    {
        let model = character::Model {
            name: "Admin".to_string(),
            level: 1,
            face: 2,
            hair_style: 2,
            x: 0,
            y: 0,
            z: 0,
            transform_id: 2,
            class_id: 1,
            race_id: Race::Human as i8,
            hair_color: 0,
            is_female: false,
            ..Default::default()
        };
        let mut active_model = create_fn(model).into_active_model();
        active_model.id = ActiveValue::NotSet;
        active_model
            .into_active_model()
            .insert(db_pool)
            .await
            .unwrap()
            .try_into_model()
            .unwrap()
    }
    #[allow(clippy::missing_panics_doc)]
    pub async fn item_factory<F>(db_pool: &DBPool, create_fn: F) -> entities::item::Model
    where
        F: FnOnce(entities::item::Model) -> entities::item::Model,
    {
        let model = entities::item::Model {
            item_id: 2,
            count: 1,
            enchant_level: 3,
            time_of_use: 0,
            loc: LocType::Paperdoll,
            loc_data: PaperDoll::RHand as i32,
            ..Default::default()
        };
        let mut active_model = create_fn(model).into_active_model();
        active_model.id = ActiveValue::NotSet;
        active_model
            .insert(db_pool)
            .await
            .unwrap()
            .try_into_model()
            .unwrap()
    }
}

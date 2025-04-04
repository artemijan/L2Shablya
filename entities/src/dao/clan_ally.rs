use crate::entities::clan_ally;
use crate::DBPool;
use sea_orm::{DbErr, EntityTrait};

#[allow(clippy::missing_errors_doc)]
impl clan_ally::Model {
    pub async fn load_all(db_pool: &DBPool) -> Result<Vec<clan_ally::Model>, DbErr> {
        clan_ally::Entity::find().all(db_pool).await
    }
}

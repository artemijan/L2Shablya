use crate::common::packets::error::PacketRun;
use entities::entities::user;
use crate::{
    common::{
        packets::{common::HandleablePacket, gs_2_ls::ChangeAccessLevel},
        traits::handlers::PacketHandler,
    },
    login_server::gs_thread::GSHandler,
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{error, info, instrument};

#[async_trait]
impl HandleablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;

    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        let user_model = user::Model::find_by_username(db_pool, &self.account).await?;
        let user_id = user_model.id;
        let mut active_model: user::ActiveModel = user_model.into();
        active_model.access_level = ActiveValue::Set(self.level);
        active_model.save(db_pool).await?;
        info!("[change access level] OK {user_id:?}");
        Ok(())
    }
}

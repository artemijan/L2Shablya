use l2_core::packets::error::PacketRun;
use entities::entities::user;
use l2_core::{
    packets::gs_2_ls::ChangeAccessLevel,
    traits::handlers::PacketHandler,
};
use crate::gs_thread::GSHandler;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;

    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool();
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

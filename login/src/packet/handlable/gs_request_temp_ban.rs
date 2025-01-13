use l2_core::shared_packets::error::PacketRun;
use entities::entities::user;
use l2_core::{
    shared_packets::gs_2_ls::RequestTempBan,
    traits::handlers::PacketHandler,
};
use crate::{
    gs_thread::GSHandler,
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for RequestTempBan {
    type HandlerType = GSHandler;
    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool();
        //ignore error updating an account
        let user_model = user::Model::find_by_username(db_pool, &self.account).await?;
        let mut active_record: user::ActiveModel = user_model.into();
        active_record.ban_duration = ActiveValue::Set(Some(self.ban_duration));
        active_record.ban_ip = ActiveValue::Set(Some(self.ip.clone()));
        active_record.save(db_pool).await?;
        info!("[Account banned] OK: {:?}", self.account);
        let lc = gs.get_controller();
        lc.update_ip_ban_list(&self.ip, self.ban_duration);
        lc.remove_player(&self.account);
        Ok(())
    }
}

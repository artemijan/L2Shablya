use crate::common::packets::error::PacketRun;
use crate::{
    common::{
        packets::{common::HandleablePacket, gs_2_ls::ChangeAccessLevel},
        traits::handlers::PacketHandler,
    },
    database::user::User,
    login_server::gs_thread::GSHandler,
};
use async_trait::async_trait;
use tracing::{error, info, instrument};

#[async_trait]
impl HandleablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;
    
    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        match User::change_access_level(db_pool, self.level, &self.account).await {
            Ok(user) => {
                info!("[change access level] OK {:?}", user.id);
            }
            Err(e) => {
                error!("[change access level] err {e:?}");
            }
        };
        Ok(())
    }
}

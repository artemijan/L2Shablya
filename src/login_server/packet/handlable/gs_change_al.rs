use async_trait::async_trait;

use crate::{
    common::{
        packets::{common::HandleablePacket, gs_2_ls::ChangeAccessLevel},
        traits::handlers::PacketHandler,
    },
    database::user::User,
    login_server::gs_thread::GSHandler,
};
use crate::common::packets::error::PacketRun;

#[async_trait]
impl HandleablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        match User::change_access_level(db_pool, self.level, &self.account).await {
            Ok(user) => {
                println!("[change access level] OK {:?}", user.id);
            }
            Err(e) => {
                println!("[change access level] err {e:?}");
            }
        };
        Ok(())
    }
}

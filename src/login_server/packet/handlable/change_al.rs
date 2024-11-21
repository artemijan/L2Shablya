use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        gs_2_ls::ChangeAccessLevel,
    }, traits::handlers::PacketHandler},
    database::user::User,
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
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
        Ok(None)
    }
}

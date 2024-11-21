use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        gs_2_ls::RequestTempBan,
    }, traits::handlers::PacketHandler},
    database::user::User,
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for RequestTempBan {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        match User::req_temp_ban(db_pool, &self.account, self.ban_duration, &self.ip).await {
            Ok(user) => {
                println!("[Account banned] OK {:?}", user.id);
            }
            Err(e) => {
                println!("[Failed to ban account] err {e:?}");
            }
        };
        let lc = gs.get_controller();
        lc.update_ip_ban_list(&self.ip, self.ban_duration);
        lc.remove_player(&self.account);
        Ok(None)
    }
}

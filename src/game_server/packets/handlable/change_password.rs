use async_trait::async_trait;

use crate::{common::packets::{common::{HandlablePacket, SendablePacket}, error, gs_2_ls::ChangePassword, ls_2_gs}, game_server::handlers::LoginHandler};


#[async_trait]
impl HandlablePacket for ChangePassword {
    type HandlerType = LoginHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        todo!()
    }
}

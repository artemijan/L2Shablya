use crate::{
    common::packets::{common::HandleablePacket, error::PacketRun, gs_2_ls::ChangePassword},
    game_server::handlers::LoginHandler,
};
use async_trait::async_trait;

#[async_trait]
impl HandleablePacket for ChangePassword {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

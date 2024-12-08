use async_trait::async_trait;

use crate::{
    common::packets::{
        common::HandleablePacket
        ,
        gs_2_ls::ChangePassword,
    },
    game_server::handlers::LoginHandler,
};
use crate::common::packets::error::PacketRun;

#[async_trait]
impl HandleablePacket for ChangePassword {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

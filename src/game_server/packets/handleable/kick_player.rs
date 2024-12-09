use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::{common::HandleablePacket, ls_2_gs::KickPlayer},
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandleablePacket for KickPlayer {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

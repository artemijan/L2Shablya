use async_trait::async_trait;

use crate::{
    common::packets::{
        common::HandleablePacket
        , ls_2_gs,
    },
    game_server::handlers::LoginHandler,
};
use crate::common::packets::error::PacketRun;

#[async_trait]
impl HandleablePacket for ls_2_gs::PlayerAuthResponse {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

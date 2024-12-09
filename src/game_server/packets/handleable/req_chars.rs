use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::{common::HandleablePacket, ls_2_gs::RequestChars},
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandleablePacket for RequestChars {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

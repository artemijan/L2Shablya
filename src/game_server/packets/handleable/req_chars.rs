use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        ls_2_gs::RequestChars,
    },
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandlablePacket for RequestChars {
    type HandlerType = LoginHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        todo!()
    }
}

use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, SendablePacket},
        error::PacketRun,
        gs_2_ls::PlayerTracert,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for PlayerTracert {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        _: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        Ok(None)
    }
}

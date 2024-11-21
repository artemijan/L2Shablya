use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, PacketType, SendablePacket},
        error::PacketRun,
        gs_2_ls::ReplyChars,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for ReplyChars {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        gs.respond_to_message(&self.account_name, PacketType::ReplyChars(self.clone()))
            .await;
        Ok(None)
    }
}

use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::{
        common::{HandleablePacket, PacketType},
        gs_2_ls::ReplyChars,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandleablePacket for ReplyChars {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        gs.respond_to_message(&self.account_name, PacketType::ReplyChars(self.clone()))
            .await;
        Ok(())
    }
}

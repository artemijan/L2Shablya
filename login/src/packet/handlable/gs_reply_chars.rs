use async_trait::async_trait;

use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use l2_core::packets::error::PacketRun;
use l2_core::packets::{common::PacketType, gs_2_ls::ReplyChars};

#[async_trait]
impl HandleablePacket for ReplyChars {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        gs.respond_to_message(&self.account_name, PacketType::ReplyChars(self.clone()))
            .await;
        Ok(())
    }
}

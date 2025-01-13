use async_trait::async_trait;

use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use l2_core::shared_packets::error::PacketRun;
use l2_core::shared_packets::{common::PacketType, gs_2_ls::ReplyChars};
use l2_core::traits::handlers::PacketHandler;

#[async_trait]
impl HandleablePacket for ReplyChars {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = gs.get_controller();
        controller.message_broker.respond_to_message(
            gs.server_id,
            &self.account_name,
            PacketType::ReplyChars(self.clone()),
        );
        Ok(())
    }
}

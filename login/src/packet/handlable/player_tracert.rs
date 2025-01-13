use async_trait::async_trait;

use crate::gs_thread::GSHandler;
use l2_core::shared_packets::{error::PacketRun, gs_2_ls::PlayerTracert};
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for PlayerTracert {
    type HandlerType = GSHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        Ok(())
    }
}

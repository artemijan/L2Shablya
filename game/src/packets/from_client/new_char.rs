use crate::client_thread::ClientHandler;
use crate::packets::to_client::NewCharacterResponse;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::packets::common::ReadablePacket;
use l2_core::packets::error::PacketRun;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct NewCharacter;

impl ReadablePacket for NewCharacter {
    fn read(_: &[u8]) -> Option<Self> {
        Some(Self {})
    }
}

#[async_trait]
impl HandleablePacket for NewCharacter {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = handler.get_controller();
        handler
            .send_packet(Box::new(NewCharacterResponse::new(controller)?))
            .await?;
        Ok(())
    }
}

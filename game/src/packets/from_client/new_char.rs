use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::packets::common::ReadablePacket;
use l2_core::packets::error::PacketRun;

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
        Ok(())
    }
}

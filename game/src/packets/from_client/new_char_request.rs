use crate::client_thread::ClientHandler;
use crate::packets::to_client::NewCharacterResponse;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::traits::handlers::PacketSender;

#[derive(Debug, Clone)]
pub struct NewCharacterRequest;

impl ReadablePacket for NewCharacterRequest {
    const PACKET_ID: u8 = 0x13;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for NewCharacterRequest {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = handler.get_controller();
        handler
            .send_packet(Box::new(NewCharacterResponse::new(controller)?))
            .await?;
        Ok(())
    }
}

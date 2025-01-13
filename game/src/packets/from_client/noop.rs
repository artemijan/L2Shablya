use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::error::PacketRun;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::info;

#[derive(Debug, Clone)]
pub struct NoOp {}

impl ReadablePacket for NoOp {
    const PACKET_ID: u8 = 0;

    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        let packet_id = buffer.read_byte();
        info!("Ignoring packet {packet_id}");
        Some(Self {})
    }
}

#[async_trait]
impl HandleablePacket for NoOp {
    type HandlerType = ClientHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        Ok(())
    }
}

use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;

#[derive(Debug, Clone)]
pub struct NoOp {}

impl ReadablePacket for NoOp {
    const PACKET_ID: u8 = 0;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for NoOp {
    type HandlerType = ClientHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> anyhow::Result<()> {
        Ok(())
    }
}

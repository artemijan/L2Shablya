use crate::client_thread::ClientHandler;
use crate::packets::to_client::extended::CharExistsResponse;
use crate::packets::utils::validate_can_create_char;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct CheckCharName {
    name: String,
}

impl ReadablePacket for CheckCharName {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0xA9);

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buff = ReadablePacketBuffer::new(data);
        Ok(Self {
            name: buff.read_string(),
        })
    }
}

#[async_trait]
impl HandleablePacket for CheckCharName {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        let pool = handler.get_db_pool();
        let reason = validate_can_create_char(pool, &self.name).await?;
        handler
            .send_packet(Box::new(CharExistsResponse::new(reason as i32)?))
            .await?;
        Ok(())
    }
}

use crate::client_thread::ClientHandler;
use crate::packets::to_client::ProtocolResponse;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::crypt::login::Encryption;
use l2_core::packets::common::ReadablePacket;
use l2_core::packets::error::PacketRun;
use l2_core::packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct ProtocolVersion {
    pub version: i32,
}

impl ReadablePacket for ProtocolVersion {
    const PACKET_ID: u8 = 0x0E;
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let version = buffer.read_i32();
        Some(Self { version })
    }
}

#[async_trait]
impl HandleablePacket for ProtocolVersion {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = handler.get_controller();
        let cfg = controller.get_cfg();
        if let Err(e) = handler.set_protocol(self.version) {
            handler
                .send_packet(Box::new(ProtocolResponse::fail(&cfg)?))
                .await?;
            return Err(PacketRun {
                msg: Some(e.to_string()),
            });
        }

        let key_bytes = ClientHandler::generate_key();
        if cfg.enable_encryption {
            let key = Encryption::from_u8_key(&key_bytes);
            handler.set_encryption(Some(key));
        }
        handler
            .send_packet(Box::new(ProtocolResponse::new(&key_bytes, true, &cfg)?))
            .await?;
        Ok(())
    }
}

use crate::client_thread::ClientHandler;
use crate::packets::to_client::extended::CharExistsResponse;
use crate::packets::utils::validate_can_create_char;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

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
            name: buff.read_c_utf16le_string()?,
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

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use super::*;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};
    use l2_core::config::gs::GSServer;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use crate::controller::Controller;

    #[tokio::test]
    pub async fn test_handle() {
        let pool = get_test_db().await;
        let pack = CheckCharName {
            name: "Test".to_string(),
        };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r,w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        let mut ch = ClientHandler::new(r,w, Ipv4Addr::LOCALHOST,pool, controller);
        pack.handle(&mut ch).await.unwrap();
        tokio::spawn(async move {
            ch.handle_client().await.unwrap();
        });
        let mut resp = [0; 9];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(resp[2], 0xFE);
        assert_eq!(i16::from_le_bytes(resp[3..=4].try_into().unwrap()), 0x10B);
    }
}

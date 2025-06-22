use crate::packets::to_client::extended::CharExistsResponse;
use crate::packets::utils::validate_can_create_char;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Debug, Clone)]
pub struct CheckCharName {
    name: String,
}

impl ReadablePacket for CheckCharName {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0xA9);

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buff = ReadablePacketBuffer::new(data);
        Ok(Self {
            name: buff.read_c_utf16le_string()?,
        })
    }
}
impl Message<CheckCharName> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: CheckCharName,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let reason = validate_can_create_char(&self.db_pool, &msg.name).await?;
        self.send_packet(
            CharExistsResponse::new(reason as i32)?.buffer,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use crate::test_utils::test::spawn_player_client_actor;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use ntest::timeout;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    #[timeout(2000)]
    pub async fn test_handle() {
        let pool = get_test_db().await;
        let pack = CheckCharName {
            name: "Test".to_string(),
        };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        let pl_actor = spawn_player_client_actor(controller, pool, r, w).await;
        let res = pl_actor.ask(pack).await;
        assert!(res.is_ok());
        let mut resp = [0; 9];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(resp[2], 0xFE);
        assert_eq!(i16::from_le_bytes(resp[3..=4].try_into().unwrap()), 0x10B);
    }
}

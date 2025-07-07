use crate::packets::to_client::ProtocolResponse;
use crate::pl_client::PlayerClient;
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::crypt::game::GameClientEncryption;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[derive(Debug, Clone)]
pub struct ProtocolVersion {
    pub version: i32,
    pub buffer: SendablePacketBuffer,
}

impl ReadablePacket for ProtocolVersion {
    const PACKET_ID: u8 = 0x0E;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let version = buffer.read_i32()?;
        Ok(Self {
            version,
            buffer: SendablePacketBuffer::empty(),
        })
    }
}
impl Message<ProtocolVersion> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ProtocolVersion,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let cfg = self.controller.get_cfg();
        if let Err(e) = self.set_protocol(msg.version) {
            self.send_packet(
                ProtocolResponse::fail(&cfg)?,
            )
            .await?;
            bail!(e);
        }

        let key_bytes = PlayerClient::generate_key();
        if cfg.enable_encryption {
            let key = GameClientEncryption::new(&key_bytes)?;
            self.set_encryption(Some(key));
        }

        self.send_packet(
            ProtocolResponse::new(&key_bytes, true, &cfg)?,
        )
        .await?;
        Ok(())
    }
}

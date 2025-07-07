use crate::packets::to_client::NewCharacterResponse;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::shared_packets::common::ReadablePacket;

#[derive(Debug, Clone)]
pub struct NewCharacterRequest;

impl ReadablePacket for NewCharacterRequest {
    const PACKET_ID: u8 = 0x13;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl Message<NewCharacterRequest> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _msg: NewCharacterRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        self.send_packet(
            NewCharacterResponse::new(&self.controller)?,
        )
        .await?;
        Ok(())
    }
}

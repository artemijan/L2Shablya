use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct NoOp {}

impl ReadablePacket for NoOp {
    const PACKET_ID: u8 = 0;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<NoOp> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: NoOp,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

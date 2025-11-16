use crate::packets::to_client::{CharSelectionInfo, RestartResponse};
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::{instrument, warn};

#[derive(Debug, Clone)]
pub struct StopMove {}

impl ReadablePacket for StopMove {
    const PACKET_ID: u8 = 0xED;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<StopMove> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: StopMove,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        warn!("TODO: stop move");
        Ok(())
    }
}

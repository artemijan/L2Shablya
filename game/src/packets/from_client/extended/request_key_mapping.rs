use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::{instrument, warn};

#[derive(Debug, Clone)]
pub struct RequestKeyMapping;

impl ReadablePacket for RequestKeyMapping {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x21);

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<RequestKeyMapping> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: RequestKeyMapping,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo:
        warn!("todo: received RequestKeyMapping");
        Ok(())
    }
}

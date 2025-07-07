use crate::packets::to_client::extended::ManorList;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RequestManorList;

impl ReadablePacket for RequestManorList {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x01);

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<RequestManorList> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _msg: RequestManorList,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let p = ManorList::new()?;
        self.send_packet(p).await?;
        Ok(())
    }
}

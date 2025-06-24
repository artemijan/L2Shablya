use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::shared_packets::common::ReadablePacket;

#[derive(Debug, Clone)]
pub struct RequestUserBanInfo;

impl ReadablePacket for RequestUserBanInfo {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x138);

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<RequestUserBanInfo> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(_msg, _ctx))]
    async fn handle(
        &mut self,
        _msg: RequestUserBanInfo,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo: I don't know what this packet is needed for, in L2J it is also not handled
        Ok(())
    }
}

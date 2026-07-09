use crate::packets::to_client::TargetUnselected;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RequestCancelTarget {
    pub target_lost: bool,
}

impl ReadablePacket for RequestCancelTarget {
    const PACKET_ID: u8 = 0x48;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            target_lost: buffer.read_i16()? == 0,
        })
    }
}

impl Message<RequestCancelTarget> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestCancelTarget,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo stop casting
        //todo: check if target is locked (aggression from tanks)
        if let Some(_) = self.selected_target.take() {
            let player = self.try_get_selected_char()?;
            self.controller
                .broadcast_packet(TargetUnselected::new(player)?);
        }
        Ok(())
    }
}

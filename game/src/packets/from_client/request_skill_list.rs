use crate::packets::to_client;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RequestSkillList;

impl ReadablePacket for RequestSkillList {
    const PACKET_ID: u8 = 0x50;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_data: BytesMut) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl Message<RequestSkillList> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _msg: RequestSkillList,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let player = self.try_get_selected_char()?;
        let packet = to_client::SkillList::new(player, &self.controller.skills)?;
        self.send_packet(packet).await?;
        Ok(())
    }
}

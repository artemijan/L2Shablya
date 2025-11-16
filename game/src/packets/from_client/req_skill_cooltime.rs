use crate::packets::to_client::SkillCoolTime;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct ReqSkillCoolTime {}

impl ReadablePacket for ReqSkillCoolTime {
    const PACKET_ID: u8 = 0xA6;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<ReqSkillCoolTime> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: ReqSkillCoolTime,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        self.send_packet(SkillCoolTime::new(self.try_get_selected_char()?)?)
            .await?;
        Ok(())
    }
}

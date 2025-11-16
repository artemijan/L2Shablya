use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::{instrument, warn};
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Debug, Clone)]
pub struct SelectedQuestZoneId{
    quest_zone_id:i32
}

impl ReadablePacket for SelectedQuestZoneId {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0xFF);

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buff = ReadablePacketBuffer::new(data);
        Ok(Self {
            quest_zone_id: buff.read_i32()?,
        })
    }
}
impl Message<SelectedQuestZoneId> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: SelectedQuestZoneId,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let char = self.try_get_selected_char_mut()?;
        char.quest_zone_id = Some(msg.quest_zone_id);
        Ok(())
    }
}

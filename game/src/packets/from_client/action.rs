use crate::packets::to_client::TargetSelected;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct Action {
    pub buffer: SendablePacketBuffer,
    pub object_id: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub origin_z: i32,
    pub action: u8,
}

impl ReadablePacket for Action {
    const PACKET_ID: u8 = 0x1F;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let object_id = buffer.read_i32()?;
        let origin_x = buffer.read_i32()?;
        let origin_y = buffer.read_i32()?;
        let origin_z = buffer.read_i32()?;
        let action = buffer.read_byte()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            object_id,
            origin_x,
            origin_y,
            origin_z,
            action,
        })
    }
}

impl Message<Action> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: Action,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let player = self.try_get_selected_char()?;
        print!("{player:?}");
        self.send_packet(TargetSelected::new(msg.object_id, 0)?)
            .await?;
        Ok(())
    }
}

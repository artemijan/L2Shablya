use crate::packets::to_client::{CharSelectionInfo, RestartResponse};
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::{instrument, warn};

#[derive(Debug, Clone)]
pub struct ValidatePosition {
    x: i32,
    y: i32,
    z: i32,
    heading: i32,
}

impl ReadablePacket for ValidatePosition {
    const PACKET_ID: u8 = 0x59;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            x: buffer.read_i32()?,
            y: buffer.read_i32()?,
            z: buffer.read_i32()?,
            heading: buffer.read_i32()?,
            // vehicle_id: buffer.read_i32()?,
        })
    }
}
impl Message<ValidatePosition> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ValidatePosition,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo: for some reason client sends it too early so selected char is not set yet
        if let Ok(char) = self.try_get_selected_char_mut(){
            //todo: check if char is teleporting or casting spells, if so, do not do anything
            warn!("todo: validate position");
            char.set_location(msg.x, msg.y, msg.z)?;
            char.set_location_heading(msg.heading);
        }
        Ok(())
    }
}

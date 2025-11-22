use crate::packets::to_client::CharMoveToLocation;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::{info, instrument};

use crate::pl_client::PlayerClient;

#[derive(Debug, Clone)]
pub struct RequestMoveToLocation {
    pub x_to: i32,
    pub y_to: i32,
    pub z_to: i32,
    pub x_from: i32,
    pub y_from: i32,
    pub z_from: i32,
}

impl ReadablePacket for RequestMoveToLocation {
    const PACKET_ID: u8 = 0x0F;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let packet = Self {
            x_to: buffer.read_i32()?,
            y_to: buffer.read_i32()?,
            z_to: buffer.read_i32()?,
            x_from: buffer.read_i32()?,
            y_from: buffer.read_i32()?,
            z_from: buffer.read_i32()?,
        };
        Ok(packet)
    }
}

impl Message<RequestMoveToLocation> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestMoveToLocation,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        info!("Received MoveToLocation packet {:?}", msg);
        //TODO check with geodata if the location is valid.
        //todo: we need to 
        {
            let selected_char = self.try_get_selected_char_mut()?;
            selected_char.set_location(msg.x_to, msg.y_to, msg.z_to)?;
        }
        let p =
            CharMoveToLocation::new(self.try_get_selected_char()?, msg.x_to, msg.y_to, msg.z_to)?;
        self.controller.broadcast_packet(p); //broadcast to all players including self
        Ok(())
    }
}

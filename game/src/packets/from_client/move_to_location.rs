use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::{info, instrument};
use entities::entities::character;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use crate::packets::from_client::char_restore::RestoreChar;
use crate::packets::from_client::char_select::SelectChar;
use crate::packets::to_client::{CharSelectionInfo, CreateCharOk};
use crate::packets::to_client::{CharMoveToLocation};

use crate::pl_client::PlayerClient;

#[derive(Debug, Clone)]
pub struct MoveToLocation {
    pub x_to: i32,
    pub y_to: i32,
    pub z_to: i32,
    pub x_from: i32,
    pub y_from: i32,
    pub z_from: i32,
}

impl ReadablePacket for MoveToLocation {
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

impl Message<MoveToLocation> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: MoveToLocation,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {

        info!("Received MoveToLocation packet {:?}", msg);
        //TODO check with geodata if the location is valid.
        let selected_char = self.try_get_selected_char();
        self.send_packet(CharMoveToLocation::new(
            selected_char.unwrap(),
            msg.x_to,
            msg.y_to,
            msg.z_to
        )?).await;

        Ok(())
    }
}
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::{instrument, warn};
use crate::movement::calculate_distance;

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
    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: RequestMoveToLocation,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //TODO check with geodata if the location is valid.

        // Get the current position for distance validation
        let player = self.try_get_selected_char()?;
        let (current_x, current_y, current_z) = (player.get_x(), player.get_y(), player.get_z());

        // Calculate distance
        let Some(distance) = calculate_distance(
            current_x, current_y, current_z, msg.x_to, msg.y_to, msg.z_to,
        ) else {
            warn!(
                "Player {} attempted movement causing coordinate overflow. Pos: ({},{},{}) -> Dest: ({},{},{})",
                player.char_model.name,
                current_x,
                current_y,
                current_z,
                msg.x_to,
                msg.y_to,
                msg.z_to
            );
            return Ok(());
        };

        // Check against max distance from config
        let cfg = self.controller.get_cfg();
        if cfg.max_movement_distance > 0 && distance > f64::from(cfg.max_movement_distance) {
            warn!(
                "Player {} attempted to move excessive distance: {:.2} (max: {})",
                player.char_model.name, distance, cfg.max_movement_distance
            );
            return Ok(());
        }

        self.start_movement(msg.x_to, msg.y_to, msg.z_to, ctx.actor_ref().clone())?;

        Ok(())
    }
}

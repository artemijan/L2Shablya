use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::{info, instrument, warn};

/// Helper function to calculate distance between two 3D points
/// Returns None if coordinate subtraction overflows
fn calculate_distance(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> Option<f64> {
    let dx = x2.checked_sub(x1)?;
    let dy = y2.checked_sub(y1)?;
    let dz = z2.checked_sub(z1)?;
    
    let dx = f64::from(dx);
    let dy = f64::from(dy);
    let dz = f64::from(dz);
    
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

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
        info!("Received MoveToLocation packet {:?}", msg);
        
        //TODO check with geodata if the location is valid.
        
        // Get current position for distance validation
        let player = self.try_get_selected_char()?;
        let (current_x, current_y, current_z) = (player.get_x(), player.get_y(), player.get_z());
        
        // Calculate distance
        let distance = match calculate_distance(current_x, current_y, current_z, msg.x_to, msg.y_to, msg.z_to) {
            Some(d) => d,
            None => {
                warn!(
                    "Player {} attempted movement causing coordinate overflow. Pos: ({},{},{}) -> Dest: ({},{},{})",
                    player.char_model.name,
                    current_x, current_y, current_z,
                    msg.x_to, msg.y_to, msg.z_to
                );
                bail!("Movement coordinates cause overflow");
            }
        };
        
        // Check against max distance from config
        let cfg = self.controller.get_cfg();
        if cfg.max_movement_distance > 0 && distance > f64::from(cfg.max_movement_distance) {
            warn!(
                "Player {} attempted to move excessive distance: {:.2} (max: {})",
                player.char_model.name,
                distance,
                cfg.max_movement_distance
            );
            return Ok(());
        }
        
        // Start or restart movement
        let (source_x, source_y, source_z) = self.start_movement(msg.x_to, msg.y_to, msg.z_to, ctx.actor_ref().clone())?;
        
        // Send initial movement packet immediately (optional, but good for responsiveness if task has delay)
        // Note: The task sends update immediately, but we might want to send the "start" packet here?
        // Actually, the previous logic removed the explicit broadcast.
        // But wait, the task sends `CharMoveToLocation`.
        // If I remove the explicit broadcast here, the task will send it.
        // So I just need to call start_movement.

        Ok(())
    }
}



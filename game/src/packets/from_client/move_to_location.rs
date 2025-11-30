use anyhow::bail;
use crate::packets::to_client::CharMoveToLocation;
use bytes::BytesMut;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use l2_core::network::connection::HandleOutboundPacket;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use std::time::Duration;
use tokio::time::interval;
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
            bail!("Movement distance exceeds maximum allowed");
        }
        
        // Start or restart movement
        let (mut movement, source_x, source_y, source_z) = self.start_movement(msg.x_to, msg.y_to, msg.z_to)?;
        
        // Send initial movement packet immediately
        let initial_packet = CharMoveToLocation::new(
            self.try_get_selected_char()?,
            msg.x_to,
            msg.y_to,
            msg.z_to,
        )?;
        self.controller.broadcast_packet(initial_packet);

        // Spawn periodic broadcast task
        let actor_ref = ctx.actor_ref().clone();
        let controller = self.controller.clone();
        let dest_x = msg.x_to;
        let dest_y = msg.y_to;
        let dest_z = msg.z_to;

        let task_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(300));
            
            loop {
                // Check movement state BEFORE waiting
                let result = actor_ref
                    .ask(GetMovementPosition)
                    .await;

                if let Ok(Some((x, y, z, has_arrived))) = result {
                    if has_arrived {
                        // Arrived at destination, stop broadcasting
                        info!("Player arrived at destination ({}, {}, {})", dest_x, dest_y, dest_z);
                        break;
                    }

                    // Broadcast current position
                    if let Ok(player) = actor_ref.ask(crate::pl_client::GetCharInfo).await {
                        if let Ok(packet) = CharMoveToLocation::new(&player, dest_x, dest_y, dest_z) {
                            controller.broadcast_packet(packet);
                        }
                    }
                } else {
                    // Error or no movement state, stop task
                    break;
                }

                // Wait for next tick
                interval.tick().await;
            }
        });

        // Store the task handle in movement state and set it in client
        movement.task_handle = Some(task_handle);
        self.movement_state = Some(movement);

        Ok(())
    }
}

/// Message to get current movement position and status
#[derive(Debug)]
pub struct GetMovementPosition;

impl Message<GetMovementPosition> for PlayerClient {
    type Reply = anyhow::Result<Option<(i32, i32, i32, bool)>>;

    async fn handle(
        &mut self,
        _: GetMovementPosition,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(movement) = self.movement_state.as_ref() {
            let (x, y, z) = movement.calculate_current_position();
            let has_arrived = movement.has_arrived();
            
            // Update player's actual position
            if let Ok(player) = self.try_get_selected_char_mut() {
                let _ = player.set_location(x, y, z);
            }
            
            Ok(Some((x, y, z, has_arrived)))
        } else {
            Ok(None)
        }
    }
}

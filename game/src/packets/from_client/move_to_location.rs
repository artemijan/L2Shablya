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
        
        // Start or restart movement (handles rerouting automatically)
        let (source_x, source_y, source_z) = self.start_movement(msg.x_to, msg.y_to, msg.z_to)?;
        
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
            let mut tick = interval(Duration::from_millis(300));
            tick.tick().await; // Skip first immediate tick

            loop {
                tick.tick().await;

                // Get current movement state via actor
                let result = actor_ref
                    .ask(GetMovementPosition)
                    .await;

                match result {
                    Ok(Some((current_x, current_y, current_z, has_arrived))) => {
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
                    }
                    Ok(None) => {
                        // Movement was stopped/cancelled
                        info!("Movement stopped");
                        break;
                    }
                    Err(e) => {
                        warn!("Error checking movement state: {}", e);
                        break;
                    }
                }
            }
        });

        // Store the task handle in movement state
        if let Some(movement) = self.movement_state.as_mut() {
            movement.task_handle = Some(task_handle);
        }

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

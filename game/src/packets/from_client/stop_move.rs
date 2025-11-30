use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::{info, instrument};

#[derive(Debug, Clone)]
pub struct StopMove {}

impl ReadablePacket for StopMove {
    const PACKET_ID: u8 = 0xED;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl Message<StopMove> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: StopMove,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        info!("Received StopMove packet");
        
        // Stop movement and get final interpolated position
        if let Some((final_x, final_y, final_z)) = self.stop_movement() {
            // Update player's location to the stopped position
            if let Ok(player) = self.try_get_selected_char_mut() {
                player.set_location(final_x, final_y, final_z)?;
            }
            info!("Movement stopped at ({}, {}, {})", final_x, final_y, final_z);
        }
        
        // TODO: Broadcast StopMove packet to nearby players once that packet is implemented
        
        Ok(())
    }
}

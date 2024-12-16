use async_trait::async_trait;

use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use l2_core::packets::error::PacketRun;
use l2_core::{packets::gs_2_ls::PlayerLogout, traits::handlers::PacketHandler};

#[async_trait]
impl HandleablePacket for PlayerLogout {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let lc = gs.get_controller();
        lc.on_player_logout(&self.acc);
        Ok(())
    }
}

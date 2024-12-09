use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::{
        packets::{common::HandleablePacket, gs_2_ls::PlayerLogout},
        traits::handlers::PacketHandler,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandleablePacket for PlayerLogout {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let lc = gs.get_controller();
        lc.on_player_logout(&self.acc);
        Ok(())
    }
}

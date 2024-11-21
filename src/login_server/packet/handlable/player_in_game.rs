use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error::PacketRun,
        gs_2_ls::PlayerInGame,
    }, traits::handlers::PacketHandler},
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for PlayerInGame {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let lc = gs.get_controller();
        lc.on_players_in_game(gs.server_id.unwrap(), &self.accounts);
        Ok(None)
    }
}

use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error::PacketRun,
        gs_2_ls::GSStatusUpdate,
    }, traits::handlers::PacketHandler},
    login_server::{
        gs_thread::GSHandler,
        packet::{login_fail::PlayerLogin, PlayerLoginFailReasons},
    },
};

#[async_trait]
impl HandlablePacket for GSStatusUpdate {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let lc = gs.get_controller();
        let mut updated = false;
        if let Some(server_id) = gs.server_id {
            updated = lc.with_gs(server_id, |gsi| {
                gsi.set_max_players(self.max_players);
                gsi.set_age_limit(self.server_age);
                gsi.use_square_brackets(self.use_square_brackets);
                gsi.set_server_type(self.server_type);
                gsi.set_server_status(self.status.clone() as i32);
            });
        }
        if !updated {
            return Err(PacketRun {
                msg: Some(format!("Server was not found, GS id {:?}", gs.server_id)),
                response: Some(Box::new(PlayerLogin::new(
                    PlayerLoginFailReasons::ReasonAccessFailed,
                ))),
            });
        }
        gs.start_channel();
        Ok(None)
    }
}

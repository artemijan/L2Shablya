use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        gs_2_ls,
        ls_2_gs::{self, AuthGS},
    }, traits::handlers::PacketHandler},
    login_server::{
        dto::game_server::GSInfo,
        gs_thread::{enums, GSHandler},
        packet::{login_fail, GSLoginFailReasons},
    },
};

#[async_trait]
impl HandlablePacket for gs_2_ls::GS {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let gsi = GSInfo::new(
            self.desired_id,
            self.accept_alternative_id,
            self.host_reserved,
            self.port,
            true,
            0,
            true,
            0,
            0,
            false,
            self.max_players,
            self.hex_id.clone(),
            &self.hosts,
        )
        .map_err(|e| error::PacketRun {
            msg: Some(e.to_string()),
            response: Some(Box::new(login_fail::GSLogin::new(GSLoginFailReasons::None))),
        })?;
        match gs.get_controller().register_gs(gsi) {
            Ok(()) => {
                gs.set_connection_state(&enums::GS::Authed)?;
                gs.server_id = Some(self.desired_id);
                Ok(Some(Box::new(AuthGS::new(self.desired_id))))
            }
            Err(e) => Err(e),
        }
    }
}

use async_trait::async_trait;

use crate::common::constants::get_server_name_by_id;
use crate::{
    common::{
        packets::{
            common::{HandleablePacket, PlayerLoginFail, PlayerLoginFailReasons},
            error::PacketRun,
            gs_2_ls::GSStatusUpdate,
        },
        traits::handlers::PacketHandler,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandleablePacket for GSStatusUpdate {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let lc = gs.get_controller();
        let mut updated = false;
        if let Some(server_id) = gs.server_id {
            updated = lc.with_gs(server_id, |gsi| {
                gsi.set_max_players(self.max_players);
                gsi.set_age_limit(self.server_age);
                gsi.use_square_brackets(self.use_square_brackets);
                gsi.set_server_type(self.server_type);
                gsi.set_server_status(self.status as i32);
            });
            println!(
                "Game server registered: {:}({server_id})",
                get_server_name_by_id(server_id).unwrap()
            );
        }
        if !updated {
            gs.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonAccessFailed,
            )))
            .await?;
            return Err(PacketRun {
                msg: Some(format!("Server was not found, GS id {:?}", gs.server_id)),
            });
        }
        gs.start_channel();
        Ok(())
    }
}

use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use l2_core::constants::get_server_name_by_id;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{
        common::{PlayerLoginFail, PlayerLoginFailReasons},
        gs_2_ls::GSStatusUpdate,
    },
    traits::handlers::PacketHandler,
};
use std::sync::Arc;
use anyhow::bail;
use tracing::{info, instrument};

#[async_trait]
impl HandleablePacket for GSStatusUpdate {
    type HandlerType = GSHandler;

    #[instrument(skip_all)]
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
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
            info!(
                "Game server registered: {:}({server_id})",
                get_server_name_by_id(server_id).unwrap()
            );
        }
        if !updated {
            gs.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonAccessFailed,
            )))
            .await?;
            bail!("Server was not found, GS id {:?}", gs.server_id);
        }
        lc.message_broker
            .register_packet_handler(gs.server_id.unwrap(), Arc::new(gs.clone()));
        Ok(())
    }
}

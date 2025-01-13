use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{
        error::PacketRun,
        gs_2_ls::PlayerAuthRequest,
        ls_2_gs::PlayerAuthResponse,
    },
    traits::handlers::PacketHandler,
};

#[async_trait]
impl HandleablePacket for PlayerAuthRequest {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let lc = gs.get_controller();
        let show_license = lc.get_config().client.show_licence;
        let operation_ok = lc.with_player(&self.account_name, |pl| {
            if let Some(session) = &pl.session {
                if session.equals(&self.session, show_license) {
                    pl.game_server = gs.server_id;
                    return true;
                }
            }
            false // operation wasn't successful
        });
        gs.send_packet(Box::new(PlayerAuthResponse::new(
            &self.account_name,
            operation_ok,
        )))
        .await?;
        Ok(())
    }
}

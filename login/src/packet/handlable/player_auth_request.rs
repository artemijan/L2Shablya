use async_trait::async_trait;
use l2_core::{
    packets::{
        error::PacketRun,
        gs_2_ls::PlayerAuthRequest,
        ls_2_gs::{KickPlayer, PlayerAuthResponse},
    },
    traits::handlers::PacketHandler,
};
use l2_core::traits::handlers::PacketSender;
use crate::{
    gs_thread::GSHandler,
};
use crate::packet::HandleablePacket;

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
        if !operation_ok {
            gs.send_packet(Box::new(KickPlayer::new(&self.account_name)))
                .await?;
            return Err(PacketRun {
                msg: Some(format!(
                    "Can't find account on LS, or the session is invalid {:}",
                    self.account_name
                )),
            });
        }
        gs.send_packet(Box::new(PlayerAuthResponse::new(&self.account_name, true)))
            .await?;
        Ok(())
    }
}

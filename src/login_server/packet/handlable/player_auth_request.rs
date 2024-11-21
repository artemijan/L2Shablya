use async_trait::async_trait;

use crate::{
    common::{packets::{
        common::{HandlablePacket, SendablePacket},
        error::PacketRun,
        gs_2_ls::PlayerAuthRequest,
        ls_2_gs::{KickPlayer, PlayerAuthResponse},
    }, traits::handlers::PacketHandler},
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for PlayerAuthRequest {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
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
            return Err(PacketRun {
                msg: Some(format!(
                    "Can't find account on LS, or the session is invalid {:}",
                    self.account_name
                )),
                response: Some(Box::new(KickPlayer::new(&self.account_name))),
            });
        }
        Ok(Some(Box::new(PlayerAuthResponse::new(
            &self.account_name,
            true,
        ))))
    }
}

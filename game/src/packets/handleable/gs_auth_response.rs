use async_trait::async_trait;
use l2_core::packets::error::PacketRun;
use l2_core::packets::gs_2_ls::{GSStatusUpdate, PlayerInGame};
use l2_core::packets::ls_2_gs;
use tracing::{info, instrument};
use l2_core::traits::handlers::PacketHandler;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for ls_2_gs::AuthGS {
    type HandlerType = LoginHandler;

    #[instrument(skip_all)]
    async fn handle(&self, lh: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = lh.get_controller();
        let cfg = controller.get_cfg();
        if self.server_id != cfg.server_id && !cfg.accept_alternative_id {
            return Err(PacketRun {
                msg: Some(format!(
                    "Can not accept alternative id from login server. Id is {}",
                    self.server_id
                )),
            });
        }
        let gsu = GSStatusUpdate::new(&cfg)?;
        lh.send_packet(Box::new(gsu)).await?;
        info!(
            "Registered on Login server: {:} ({:})",
            self.server_name, self.server_id
        );
        let accounts = controller.get_online_accounts();
        if !accounts.is_empty() {
            lh.send_packet(Box::new(PlayerInGame::new(accounts)?))
                .await?;
        }
        Ok(())
    }
}

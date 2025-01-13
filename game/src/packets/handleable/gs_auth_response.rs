use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::error::PacketRun;
use l2_core::shared_packets::gs_2_ls::{GSStatusUpdate, PlayerInGame};
use l2_core::shared_packets::ls_2_gs;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use std::sync::Arc;
use tracing::{info, instrument};

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
        controller
            .message_broker
            .register_packet_handler(Self::HandlerType::HANDLER_ID, Arc::new(lh.clone()));
        let accounts = controller.get_online_accounts();
        if !accounts.is_empty() {
            lh.send_packet(Box::new(PlayerInGame::new(&accounts)?))
                .await?;
        }
        Ok(())
    }
}

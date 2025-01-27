use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::ls_2_gs::KickPlayer;
use l2_core::traits::handlers::PacketHandler;

#[async_trait]
impl HandleablePacket for KickPlayer {
    type HandlerType = LoginHandler;
    async fn handle(&self, ph: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = ph.get_controller();
        controller.logout_account(&self.account_name);
        Ok(())
    }
}

use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::packets::error::PacketRun;
use l2_core::packets::ls_2_gs::KickPlayer;
use l2_core::traits::handlers::PacketHandler;

#[async_trait]
impl HandleablePacket for KickPlayer {
    type HandlerType = LoginHandler;
    async fn handle(&self, ph: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = ph.get_controller();
        controller.remove_online_account(&self.account_name);
        Ok(())
    }
}

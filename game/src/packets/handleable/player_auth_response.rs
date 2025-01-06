use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::packets::common::PacketType;
use l2_core::packets::error::PacketRun;
use l2_core::packets::ls_2_gs;
use l2_core::traits::handlers::PacketHandler;

#[async_trait]
impl HandleablePacket for ls_2_gs::PlayerAuthResponse {
    type HandlerType = LoginHandler;
    async fn handle(&self, ph: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = ph.get_controller();
        controller.message_broker.respond_to_message(
            Some(Self::HandlerType::HANDLER_ID),
            &self.account,
            PacketType::PlayerAuthResp(self.clone()),
        );
        Ok(())
    }
}

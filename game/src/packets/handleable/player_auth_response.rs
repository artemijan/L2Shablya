use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::PacketType;
use l2_core::shared_packets::ls_2_gs;
use l2_core::traits::handlers::PacketHandler;

#[async_trait]
impl HandleablePacket for ls_2_gs::PlayerAuthResponse {
    type HandlerType = LoginHandler;
    async fn handle(&self, ph: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = ph.get_controller();
        controller.message_broker.respond_to_message(
            Some(Self::HandlerType::HANDLER_ID),
            &self.account,
            PacketType::PlayerAuthResp(self.clone()),
        );
        Ok(())
    }
}

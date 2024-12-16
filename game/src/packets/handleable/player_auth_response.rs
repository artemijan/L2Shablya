use async_trait::async_trait;
use l2_core::packets::ls_2_gs;
use l2_core::packets::error::PacketRun;
use crate::{
    handlers::LoginHandler,
};
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for ls_2_gs::PlayerAuthResponse {
    type HandlerType = LoginHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

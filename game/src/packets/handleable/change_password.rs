use l2_core::packets::{error::PacketRun, gs_2_ls::ChangePassword};
use crate::{
    handlers::LoginHandler,
};
use async_trait::async_trait;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for ChangePassword {
    type HandlerType = LoginHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

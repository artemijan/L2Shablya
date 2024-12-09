use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::{common::HandleablePacket, gs_2_ls::ChangePassword},
    login_server::gs_thread::GSHandler,
};
use async_trait::async_trait;

#[async_trait]
impl HandleablePacket for ChangePassword {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        todo!()
    }
}

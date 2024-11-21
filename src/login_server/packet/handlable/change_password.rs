use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        gs_2_ls::ChangePassword,
    },
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandlablePacket for ChangePassword {
    type HandlerType = GSHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        todo!()
    }
}

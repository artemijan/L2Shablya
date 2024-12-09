use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::{common::HandleablePacket, gs_2_ls::PlayerTracert},
    login_server::gs_thread::GSHandler,
};

#[async_trait]
impl HandleablePacket for PlayerTracert {
    type HandlerType = GSHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        Ok(())
    }
}

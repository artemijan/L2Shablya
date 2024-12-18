use async_trait::async_trait;
use l2_core::packets::common::GSLoginFail;
use l2_core::packets::error::PacketRun;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for GSLoginFail {
    type HandlerType = LoginHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        Err(PacketRun {
            msg: Some(format!(
                "Failed to register on Login server{:?}",
                self.reason
            )),
        })
    }
}

use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::GSLoginFail;

#[async_trait]
impl HandleablePacket for GSLoginFail {
    type HandlerType = LoginHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> anyhow::Result<()> {
        bail!("Failed to register on Login server{:?}", self.reason)
    }
}

use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::traits::Shutdown;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Logout;

impl ReadablePacket for Logout {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for Logout {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo handle proper logout mechanism: olympiad,
        // in battle state, on RB and so on, offline trade, etc...
        info!("Player logged out: {:?}", handler.try_get_user()?);
        handler.get_shutdown_listener().notify_one();
        Ok(())
    }
}

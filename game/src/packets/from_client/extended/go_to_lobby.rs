use crate::client_thread::ClientHandler;
use crate::packets::to_client::CharSelectionInfo;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct GoLobby;

impl ReadablePacket for GoLobby {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x33);

    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for GoLobby {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        let user_name = &handler.try_get_user()?.username;
        let sk = handler
            .get_session_key()
            .ok_or(anyhow::anyhow!("Can not go to lobby, Session is missing"))?;
        let chars = handler
            .get_account_chars()
            .ok_or(anyhow::anyhow!("Can not go to lobby, no chars were set"))?;
        let controller = handler.get_controller();
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}

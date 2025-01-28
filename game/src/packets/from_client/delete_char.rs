use crate::client_thread::ClientHandler;
use crate::packets::to_client::CharSelectionInfo;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use entities::entities::character;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct DeleteChar {
    char_slot: i32,
}

impl ReadablePacket for DeleteChar {
    const PACKET_ID: u8 = 0x0D;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        Ok(Self {
            char_slot: buffer.read_i32(),
        })
    }
}

#[async_trait]
impl HandleablePacket for DeleteChar {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo handle proper deletion checks, e.g. check in clan, war, and so on
        let db_pool = handler.get_db_pool().clone();
        handler
            .with_char_by_slot_id(self.char_slot, |model| async move {
                character::Model::delete_char(&db_pool, model).await
            })
            .await?;
        let sk = handler.get_session_key().ok_or(anyhow::anyhow!(
            "Error after char deletion, Session is missing"
        ))?;
        let chars = handler.get_account_chars().ok_or(anyhow::anyhow!(
            "Programming error, seems like all chars dropped from the list during deletion"
        ))?;
        let controller = handler.get_controller();
        let user_name = &handler
            .user
            .as_ref()
            .ok_or(anyhow::anyhow!(
                "Programming error, or possible cheating: missing user in handler"
            ))?
            .username;
        let p = CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}

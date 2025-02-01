use crate::client_thread::ClientHandler;
use crate::packets::to_client::CharSelectionInfo;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::entities::character;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct RestoreChar {
    char_slot: i32,
}

impl ReadablePacket for RestoreChar {
    const PACKET_ID: u8 = 0x7B;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32(),
        })
    }
}

#[async_trait]
impl HandleablePacket for RestoreChar {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo flood protection
        let db_pool = handler.get_db_pool().clone();
        handler
            .with_char_by_slot_id(self.char_slot, |model| async move {
                if model.delete_at.is_none() {
                    bail!("Possible cheat: The char can't be restored as it is not deleted");
                }
                let new_char = character::Model::restore_char(&db_pool, model).await?;
                Ok(new_char)
            })
            .await?;
        let sk = handler.try_get_session_key()?;
        let chars = handler.try_get_account_chars()?;
        let controller = handler.get_controller();
        let user_name = &handler.try_get_user()?.username;
        let p = CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}

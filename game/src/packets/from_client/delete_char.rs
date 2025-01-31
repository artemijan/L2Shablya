use crate::client_thread::ClientHandler;
use crate::packets::to_client::{CharDeleteFail, CharSelectionInfo};
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::entities::character;
use l2_core::enums::CharDeletionFailReasons;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use sea_orm::DbErr;
use tracing::error;

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
        let deletion_result = handler
            .with_char_by_slot_id(self.char_slot, |model| async move {
                if model.delete_at.is_some() {
                    bail!("Possible cheating: Char already set to be deleted")
                }
                let new_char = character::Model::delete_char(&db_pool, model).await?;
                Ok(new_char)
            })
            .await;
        if let Err(err) = deletion_result {
            return if let Some(db_err) = err.downcast_ref::<DbErr>() {
                let packet = CharDeleteFail::new(CharDeletionFailReasons::Unknown)?;
                error!("DB error while deleting a character {db_err:?}");
                handler.send_packet(Box::new(packet)).await
            } else {
                Err(err)
            };
        }
        let sk = handler.try_get_session_key()?;
        let chars = handler.try_get_account_chars()?;
        let controller = handler.get_controller();
        let user_name = &handler.try_get_user()?.username;
        let p = CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}

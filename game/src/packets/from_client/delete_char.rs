use crate::packets::to_client::{CharDeleteFail, CharSelectionInfo};
use crate::pl_client::PlayerClient;
use anyhow::bail;
use bytes::BytesMut;
use entities::entities::character;
use kameo::message::{Context, Message};
use l2_core::enums::CharDeletionFailReasons;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use sea_orm::DbErr;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
pub struct DeleteChar {
    char_slot: i32,
}

impl ReadablePacket for DeleteChar {
    const PACKET_ID: u8 = 0x0D;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
    }
}
impl Message<DeleteChar> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: DeleteChar,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo handle proper deletion checks, e.g. check in clan, war, and so on
        let pool = self.db_pool.clone();
        let deletion_result = self
            .with_char_by_slot_id(msg.char_slot, |model| async move {
                if model.delete_at.is_some() {
                    bail!("Possible cheating: Char already set to be deleted")
                }
                let new_char = character::Model::delete_char(&pool, model).await?;
                Ok(new_char)
            })
            .await;
        if let Err(err) = deletion_result {
            return if let Some(db_err) = err.downcast_ref::<DbErr>() {
                let packet = CharDeleteFail::new(CharDeletionFailReasons::Unknown)?;
                error!("DB error while deleting a character {db_err:?}");
                self.send_packet(packet.buffer).await
            } else {
                Err(err)
            };
        }
        let sk = self.try_get_session_key()?;
        let chars = self.try_get_account_chars()?;
        let user_name = &self.try_get_user()?.username;
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), &self.controller, chars)?;
        self.send_packet( p.buffer).await?;
        Ok(())
    }
}

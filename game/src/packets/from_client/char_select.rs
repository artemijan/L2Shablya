use crate::client_thread::{ClientHandler, ClientStatus};
use crate::packets::to_client::CharSelected;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct SelectChar {
    char_slot: i32,
}

impl ReadablePacket for SelectChar {
    const PACKET_ID: u8 = 0x12;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32(),
        })
        //unknown part
        //buffer.read_i16()
        //buffer.read_i32()
        //buffer.read_i32()
    }
}

#[async_trait]
impl HandleablePacket for SelectChar {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo flood protection, ban check, maybe set online status in DB?
        if handler.get_selected_char_slot().is_some() {
            bail!("Char already selected")
        }
        let game_time = handler.get_controller().get_game_time();
        handler.set_status(ClientStatus::Entering);
        handler.select_char(self.char_slot);
        let char_info = handler.try_get_char_by_slot_id(self.char_slot)?;
        handler
            .send_packet(Box::new(CharSelected::new(
                char_info,
                handler.try_get_session_key()?.get_play_session_id(),
                game_time
            )?))
            .await?;
        Ok(())
    }
}

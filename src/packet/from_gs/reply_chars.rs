use crate::login_server::gs_thread::GSHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::{GSHandle, PacketType};
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use async_trait::async_trait;
use crate::login_server::traits::PacketHandler;

#[derive(Clone, Debug)]
pub struct ReplyChars {
    pub account_name: String,
    pub chars: u8,
    pub chars_to_delete: u8,
    pub char_list: Vec<i64>,
}

#[async_trait]
impl ReadablePacket for ReplyChars {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        let chars = buffer.read_byte();
        let chars_to_delete = buffer.read_byte();
        let mut char_list: Vec<i64> = vec![];
        for _ in 0..chars_to_delete {
            char_list.push(buffer.read_i64());
        }
        Some(ReplyChars {
            account_name,
            chars,
            chars_to_delete,
            char_list,
        })
    }
}

#[async_trait]
impl GSHandle for ReplyChars {
    async fn handle(&self, gs: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        gs.respond_to_message(
            &self.account_name, 
            PacketType::ReplyChars(self.clone()),
        ).await;
        Ok(None)
    }
}

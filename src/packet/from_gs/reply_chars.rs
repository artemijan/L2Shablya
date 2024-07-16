use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::{GSHandle};
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRunError;
use async_trait::async_trait;
use crate::login_server::gs_handler::GSHandler;

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
            char_list.push(buffer.read_i64())
        }
        Some(ReplyChars {
            account_name,
            char_list,
            chars,
            chars_to_delete,
        })
    }
}

#[async_trait]
impl GSHandle for ReplyChars {
    async fn handle(
        &self,
        _: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRunError> {
        Ok(None)
    }
}


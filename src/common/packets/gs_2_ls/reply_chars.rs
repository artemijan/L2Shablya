use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;
use async_trait::async_trait;

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
        Some(Self {
            account_name,
            chars,
            chars_to_delete,
            char_list,
        })
    }
}

use crate::packets::common::{ReadablePacket, SendablePacket};
use crate::packets::read::ReadablePacketBuffer;
use crate::packets::write::SendablePacketBuffer;
use async_trait::async_trait;
use entities::entities::character;

#[derive(Clone, Debug)]
pub struct ReplyChars {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
    pub chars: u8,
    pub delete_chars_len: u8,
    pub char_deletion_timestamps: Vec<i64>,
}

impl ReplyChars {
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(account_name: String, chars: &[character::Model]) -> ReplyChars {
        let mut chars_to_del_list = vec![];
        for ch in chars {
            if let Some(del_at) = ch.delete_at {
                chars_to_del_list.push(del_at.timestamp());
            }
        }
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            account_name,
            chars: 0,
            delete_chars_len: 0,
            char_deletion_timestamps: chars_to_del_list,
        };
        inst.buffer.write(0x08).unwrap();
        inst.buffer.write_string(Some(&inst.account_name)).unwrap();
        inst.buffer.write(inst.chars).unwrap();
        inst.buffer.write(inst.delete_chars_len).unwrap();
        for ch in &inst.char_deletion_timestamps {
            inst.buffer.write_i64(*ch).unwrap();
        }
        inst
    }
}

#[async_trait]
impl ReadablePacket for ReplyChars {
    const PACKET_ID: u8 = 0x08;

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
            buffer: SendablePacketBuffer::empty(),
            account_name,
            chars,
            delete_chars_len: chars_to_delete,
            char_deletion_timestamps: char_list,
        })
    }
}

#[async_trait]
impl SendablePacket for ReplyChars {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

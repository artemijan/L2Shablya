use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use async_trait::async_trait;
use bytes::BytesMut;
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
    #[allow(clippy::cast_possible_truncation, clippy::missing_errors_doc)]
    pub fn new(account_name: String, chars: &[character::Model]) -> anyhow::Result<ReplyChars> {
        let mut chars_to_del_list = vec![];
        for ch in chars {
            if let Some(del_at) = ch.delete_at {
                chars_to_del_list.push(del_at.timestamp());
            }
        }
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            account_name,
            chars: chars.len() as u8,
            delete_chars_len: chars_to_del_list.len() as u8,
            char_deletion_timestamps: chars_to_del_list,
        };
        inst.buffer.write(0x08)?;
        inst.buffer
            .write_c_utf16le_string(Some(&inst.account_name))?;
        inst.buffer.write(inst.chars)?;
        inst.buffer.write(inst.delete_chars_len)?;
        for ch in &inst.char_deletion_timestamps {
            inst.buffer.write_i64(*ch)?;
        }
        Ok(inst)
    }
}

#[async_trait]
impl ReadablePacket for ReplyChars {
    const PACKET_ID: u8 = 0x08;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data:BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_c_utf16le_string()?;
        let chars = buffer.read_byte()?;
        let chars_to_delete = buffer.read_byte()?;
        let mut char_list: Vec<i64> = vec![];
        for _ in 0..chars_to_delete {
            char_list.push(buffer.read_i64()?);
        }
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
            chars,
            delete_chars_len: chars_to_delete,
            char_deletion_timestamps: char_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::types::chrono;

    #[test]
    fn test_reply_chars() {
        let acc_name = [116, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let chars = 2;
        let del_chars = 1;
        let del_char = Utc::now().timestamp();
        let mut data = vec![0x08];
        data.extend(&acc_name);
        data.push(chars);
        data.push(del_chars);
        data.extend(&del_char.to_le_bytes());
        let packet = ReplyChars::read(BytesMut::from(&data[..])).unwrap();
        assert_eq!(packet.account_name, "test");
        assert_eq!(packet.chars, 2);
        assert_eq!(packet.delete_chars_len, 1);
        assert_eq!(packet.char_deletion_timestamps[0], del_char);
    }
    #[test]
    fn test_reply_chars_new() {
        let acc_name = "betterTest".to_string();
        let chars = vec![
            character::Model {
                id: 1,
                ..Default::default()
            },
            character::Model {
                id: 2,
                delete_at: Some(
                    "2025-01-01T12:00:00Z"
                        .parse::<DateTime<Utc>>()
                        .unwrap()
                        .into(),
                ),
                ..Default::default()
            },
        ];
        let mut packet = ReplyChars::new(acc_name, &chars).unwrap();
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(
            data,
            [
                35, 0, 8, 98, 0, 101, 0, 116, 0, 116, 0, 101, 0, 114, 0, 84, 0, 101, 0, 115, 0,
                116, 0, 0, 0, 2, 1, 64, 46, 117, 103, 0, 0, 0, 0
            ]
        );
    }
}

use crate::packets::common::ReadablePacket;
use crate::packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct ChangePassword {
    pub account: String,
    pub char_name: String,
    pub current_password: String,
    pub new_password: String,
}

impl ReadablePacket for ChangePassword {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            account: buffer.read_string(),
            char_name: buffer.read_string(),
            current_password: buffer.read_string(),
            new_password: buffer.read_string(),
        })
    }
}

use crate::packets::common::ReadablePacket;
use crate::packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct ChangeAL {
    pub account: String,
    pub level: i32,
}

impl ReadablePacket for ChangeAL {
    const PACKET_ID: u8 = 0x04;

    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            level: buffer.read_i32(),
            account: buffer.read_string(),
        })
    }
}

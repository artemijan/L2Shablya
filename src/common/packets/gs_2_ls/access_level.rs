use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;

#[derive(Clone, Debug)]
pub struct ChangeAL {
    pub account: String,
    pub level: i32,
}

impl ReadablePacket for ChangeAL {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            level: buffer.read_i32(),
            account: buffer.read_string(),
        })
    }
}

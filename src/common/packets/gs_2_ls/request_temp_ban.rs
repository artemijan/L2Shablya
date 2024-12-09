use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;

#[derive(Clone, Debug)]
pub struct RequestTempBan {
    pub account: String,
    pub ban_duration: i64,
    pub ip: String,
}

impl ReadablePacket for RequestTempBan {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            account: buffer.read_string(),
            ip: buffer.read_string(),
            ban_duration: buffer.read_i64(),
        })
    }
}

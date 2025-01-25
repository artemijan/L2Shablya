use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct RequestTempBan {
    pub account: String,
    pub ban_duration: i64,
    pub ip: String,
}

impl ReadablePacket for RequestTempBan {
    const PACKET_ID: u8 = 0x0A;
const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Ok(Self {
            account: buffer.read_string(),
            ip: buffer.read_string(),
            ban_duration: buffer.read_i64(),
        })
    }
}

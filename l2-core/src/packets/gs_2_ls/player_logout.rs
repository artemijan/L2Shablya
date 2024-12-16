use crate::packets::common::ReadablePacket;
use crate::packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct PlayerLogout {
    pub acc: String,
}

impl ReadablePacket for PlayerLogout {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let acc = buffer.read_string();
        Some(Self { acc })
    }
}

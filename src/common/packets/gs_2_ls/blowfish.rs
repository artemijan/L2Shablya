use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::packets::error::PacketRun;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct BlowFish {
    pub encrypted_key: Vec<u8>,
}

impl ReadablePacket for BlowFish {
    #[allow(clippy::cast_sign_loss)]
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i32();
        Some(Self {
            encrypted_key: buffer.read_bytes(size as usize),
        })
    }
}

use crate::shared_packets::common::{ReadablePacket, SendablePacket};
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;
use crate as l2_core;

#[derive(Clone, Debug, SendablePacketImpl)]
pub struct BlowFish {
    buffer: SendablePacketBuffer,
    pub encrypted_key: Vec<u8>,
}
impl BlowFish {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(encrypted_key: Vec<u8>) -> Self {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            encrypted_key,
        };
        inst.write_all()
            .expect("Failed to write bytes while building blowfish packet.");
        inst
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x00)?;
        self.buffer.write_u32(self.encrypted_key.len() as u32)?;
        self.buffer.write_bytes(&self.encrypted_key)?;
        Ok(())
    }
}
impl ReadablePacket for BlowFish {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;

    #[allow(clippy::cast_sign_loss)]
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i32();
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            encrypted_key: buffer.read_bytes(size as usize),
        })
    }
}

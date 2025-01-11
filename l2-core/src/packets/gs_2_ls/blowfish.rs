use crate::packets::common::{ReadablePacket, SendablePacket};
use crate::packets::read::ReadablePacketBuffer;
use crate::packets::write::SendablePacketBuffer;

#[derive(Clone, Debug)]
pub struct BlowFish {
    buffer: SendablePacketBuffer,
    pub encrypted_key: Vec<u8>,
}
impl BlowFish {
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

    #[allow(clippy::cast_sign_loss)]
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i32();
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            encrypted_key: buffer.read_bytes(size as usize),
        })
    }
}
impl SendablePacket for BlowFish {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

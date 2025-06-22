use bytes::BytesMut;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;

#[derive(Clone, Debug)]
pub struct BlowFish {
    pub buffer: SendablePacketBuffer,
    pub encrypted_key: BytesMut,
}
impl BlowFish {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(encrypted_key: BytesMut) -> Self {
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
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let size = buffer.read_i32()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            encrypted_key: BytesMut::from(buffer.read_bytes(size as usize)?),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_packets::common::SendablePacket;
    fn get_bytes() -> [u8; 34] {
        [
            34, 0, 0, 16, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }
    #[test]
    fn blowfish_new() {
        let mut buffer = BlowFish::new(BytesMut::from(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]));
        buffer.buffer.write_padding().unwrap();
        let bytes = buffer.buffer.take();
        assert_eq!(bytes.len(), 34);
        assert_eq!(bytes.as_ref(), get_bytes());
    }
    #[test]
    fn blowfish_read() {
        let pack = BlowFish::read(BytesMut::from(&get_bytes()[2..])).unwrap();
        assert_eq!(pack.encrypted_key, &get_bytes()[7..7 + 16]);
    }
}

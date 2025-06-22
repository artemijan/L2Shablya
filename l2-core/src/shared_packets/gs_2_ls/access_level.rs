use bytes::BytesMut;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct ChangeAL {
    pub account: String,
    pub level: i32,
}

impl ReadablePacket for ChangeAL {
    const PACKET_ID: u8 = 0x04;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        Ok(Self {
            level: buffer.read_i32()?,
            account: buffer.read_c_utf16le_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use crate::shared_packets::common::ReadablePacket;
    use crate::shared_packets::gs_2_ls::access_level::ChangeAL;

    #[test]
    fn test_read() {
        let text_bytes: Vec<u8> = "test"
            .encode_utf16()
            .flat_map(u16::to_le_bytes) // Convert each `u16` to little-endian bytes
            .collect();
        let mut data = BytesMut::from(&[0x04, 69, 0, 0, 0][..]);
        data.extend(text_bytes);
        let p = ChangeAL::read(data).unwrap();
        assert_eq!(p.account, "test");
        assert_eq!(p.level, 69);
    }
}

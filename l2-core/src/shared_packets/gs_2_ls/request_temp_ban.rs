use bytes::BytesMut;
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

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        Ok(Self {
            account: buffer.read_c_utf16le_string()?,
            ip: buffer.read_c_utf16le_string()?,
            ban_duration: buffer.read_i64()?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_request_temp_ban() {
        let ban_duration: i64 = 5;
        let mut data = vec![0x0A];
        data.extend([116, 0, 101, 0, 115, 0, 116, 0, 0, 0]);
        data.extend(b"1\x009\x002\x00.\x001\x006\x008\x00.\x000\x00.\x001\x000\x000\x00\x00\x00");
        data.extend(ban_duration.to_le_bytes());
        let packet = RequestTempBan::read(BytesMut::from(&data[..])).unwrap();
        assert_eq!(packet.account, "test");
        assert_eq!(packet.ip, "192.168.0.100");
        assert_eq!(packet.ban_duration, 5);
    }
}

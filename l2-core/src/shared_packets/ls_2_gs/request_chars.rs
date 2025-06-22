use crate::shared_packets::{
    common::ReadablePacket, read::ReadablePacketBuffer, write::SendablePacketBuffer,
};
use bytes::BytesMut;

#[derive(Debug, Clone)]
pub struct RequestChars {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
}

impl RequestChars {
    #[must_use]
    pub fn new(account_name: &str) -> RequestChars {
        let mut gg = RequestChars {
            buffer: SendablePacketBuffer::new(),
            account_name: account_name.to_string(),
        };
        let _ = gg.write_all(); // safe to ignore
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x05)?;
        self.buffer
            .write_c_utf16le_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl ReadablePacket for RequestChars {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_c_utf16le_string()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_request_chars() {
        let acc = "test";
        let mut packet = RequestChars::new(acc);
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(data, [13, 0, 5, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0]);
    }
    #[test]
    fn test_request_chars_read() {
        let buff = BytesMut::from(&[5, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0][..]);
        let packet = RequestChars::read(buff).unwrap();
        assert_eq!(packet.account_name, "test");
    }
}

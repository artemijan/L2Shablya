use crate as l2_core;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Clone, Debug, SendablePacketImpl)]
pub struct ChangePassword {
    pub account: String,
    pub char_name: String,
    pub current_password: String,
    pub new_password: String,
    buffer: SendablePacketBuffer,
}

impl ReadablePacket for ChangePassword {
    const PACKET_ID: u8 = 0x0B;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account: buffer.read_c_utf16le_string()?,
            char_name: buffer.read_c_utf16le_string()?,
            current_password: buffer.read_c_utf16le_string()?,
            new_password: buffer.read_c_utf16le_string()?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_password_read() {
        let acc = [ 116, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let char_name = [ 111, 0, 101, 0, 113, 0, 118, 0, 0, 0];
        let current_password = [ 110, 0, 119, 0, 112, 0, 117, 0, 0, 0];
        let new_password = [ 112, 0, 119, 0, 116, 0, 118, 0, 0, 0];
        let mut data = vec![0x0B];
        data.extend_from_slice(&acc);
        data.extend_from_slice(&char_name);
        data.extend_from_slice(&current_password);
        data.extend_from_slice(&new_password);
        let packet = ChangePassword::read(&data).unwrap();
        assert_eq!(packet.account, "test");
        assert_eq!(packet.char_name, "oeqv");
        assert_eq!(packet.current_password, "nwpu");
        assert_eq!(packet.new_password, "pwtv");

    }
}
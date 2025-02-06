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

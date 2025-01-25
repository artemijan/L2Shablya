use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct ChangePassword {
    pub account: String,
    pub char_name: String,
    pub current_password: String,
    pub new_password: String,
}

impl ReadablePacket for ChangePassword {
    const PACKET_ID: u8 = 0x0B;
const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Ok(Self {
            account: buffer.read_string(),
            char_name: buffer.read_string(),
            current_password: buffer.read_string(),
            new_password: buffer.read_string(),
        })
    }
}

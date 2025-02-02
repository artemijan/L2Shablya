use crate as l2_core;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Clone, Debug, SendablePacketImpl)]
pub struct PlayerLogout {
    pub acc: String,
    pub buffer: SendablePacketBuffer,
}
impl PlayerLogout {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(acc: &str) -> anyhow::Result<Self> {
        let mut inst = Self {
            acc: String::new(),
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_c_utf16le_string(Some(acc))?;
        Ok(inst)
    }
}
impl ReadablePacket for PlayerLogout {
    const PACKET_ID: u8 = 0x03;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let acc = buffer.read_c_utf16le_string()?;
        Ok(Self {
            acc,
            buffer: SendablePacketBuffer::empty(),
        })
    }
}

use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct UnreadMailCount {
    pub(crate) buffer: SendablePacketBuffer,
}

impl UnreadMailCount {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x13C;
    pub fn new(count: u32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_u32(count)?;
        Ok(inst)
    }
}

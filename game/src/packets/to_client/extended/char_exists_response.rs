use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct CharExistsResponse {
    buffer: SendablePacketBuffer,
    allow_response: i32,
}

impl CharExistsResponse {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x10B;
    pub fn new(allow_response: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            allow_response,
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(inst.allow_response)?;
        Ok(inst)
    }
}

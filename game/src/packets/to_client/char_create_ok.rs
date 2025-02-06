use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct CreateCharOk {
    buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CreateCharOk {
    const PACKET_ID: u8 = 0x0F;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(1)?;
        Ok(inst)
    }
}

use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct AutoSoulShots {
    pub(crate) buffer: SendablePacketBuffer,
}

impl AutoSoulShots {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x0C;

    pub fn new(item_id:i32, enable:bool, the_type:i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(item_id)?;
        inst.buffer.write_i32(enable)?;
        inst.buffer.write_i32(the_type)?;
        Ok(inst)
    }
}

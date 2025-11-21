use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct DeleteObject {
    pub(crate) buffer: SendablePacketBuffer,
}

impl DeleteObject {
    const PACKET_ID: u8 = 0x08;

    pub fn new(obj_id: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(obj_id)?;
        inst.buffer.write(0)?; // c2
        Ok(inst)
    }
}

use l2_core::enums::CharDeletionFailReasons;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct CharDeleteFail {
    buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CharDeleteFail {
    const PACKET_ID: u8 = 0x1E;
    const EX_PACKET_ID: Option<u16> = None;

    pub fn new(reason: CharDeletionFailReasons) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(reason as i32)?;
        Ok(inst)
    }
}

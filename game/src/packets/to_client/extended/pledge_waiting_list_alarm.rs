use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct PledgeWaitingListAlarm {
    pub(crate) buffer: SendablePacketBuffer,
}

impl PledgeWaitingListAlarm {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x147;
    pub fn new() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        Ok(inst)
    }
}

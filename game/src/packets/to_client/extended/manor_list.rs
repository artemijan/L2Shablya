use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct ManorList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl ManorList {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x22;
    pub fn new() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        //todo: implement me
        inst.buffer.write_i32(0)?; //size
        Ok(inst)
    }
}

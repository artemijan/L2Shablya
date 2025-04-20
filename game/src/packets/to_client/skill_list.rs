use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct SkillList {
    buffer: SendablePacketBuffer,
}

impl SkillList {
    const PACKET_ID: u8 = 0x5F;
    pub fn empty() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write_i32(0)?; //skill count
        inst.buffer.write_i32(0)?;
        Ok(inst)
    }
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let last_learned_skill = 0;
        //todo: implement me
        inst.buffer.write_i32(0)?; //skill count
        inst.buffer.write_i32(last_learned_skill)?;
        Ok(inst)
    }
}

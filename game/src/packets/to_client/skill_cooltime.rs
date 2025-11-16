use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;
use tracing::log::warn;

#[derive(Debug, Clone, SendablePacket)]
pub struct SkillCoolTime {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillCoolTime {
    const PACKET_ID: u8 = 0xC7;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(0)?; // timestamps count
        //todo: implement cache for skill cooltime
        warn!("TODO: implement ReqSkillCoolTime packet");
        Ok(inst)
    }
}
#[cfg(test)]
mod test {}

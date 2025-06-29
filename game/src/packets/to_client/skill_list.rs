use entities::entities::skill;
use entities::DBPool;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;
use l2_core::game_objects::creature::skill::Skill;

#[derive(Debug, Clone)]
pub struct SkillList {
    pub(crate) buffer: SendablePacketBuffer,
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
    pub async fn new(pool: &DBPool, p: &mut Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let last_learned_skill = 0;
        if p.skills.is_none() {
            let skills = Skill::load_for_char(pool, p.char_model.id).await?;
            p.skills = Some(skills);
        }
        if let Some(skills) = p.skills.as_ref() {
            inst.buffer.write_u32(u32::try_from(skills.len())?)?; //skill count
            for sk in skills{
                inst.buffer.write_i32(sk.passive)?;
                inst.buffer.write_i16(sk.model.level)?;
                inst.buffer.write_i16(sk.model.sub_level)?;
                inst.buffer.write_i32(sk.model.id)?;
                inst.buffer.write_i32(sk.reuse_delay_group)?;
                inst.buffer.write(sk.disabled)?;
                inst.buffer.write(sk.can_enchant)?;
            }
        } else {
            inst.buffer.write_u32(0u32)?; //skill count
        }
        inst.buffer.write_i32(last_learned_skill)?;
        Ok(inst)
    }
}

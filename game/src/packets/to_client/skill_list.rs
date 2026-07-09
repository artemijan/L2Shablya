use l2_core::data::skills::SkillsData;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Debug, Clone, SendablePacket)]
pub struct SkillList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillList {
    pub const PACKET_ID: u8 = 0x5F;

    pub fn new(player: &Player, skills_data: &SkillsData) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };

        inst.buffer.write(Self::PACKET_ID)?;

        if let Some(skills) = &player.skills {
            inst.buffer.write_i32(skills.len() as i32)?;
            for skill in skills {
                let reuse_group = skills_data
                    .get_skill(skill.model.id as u32, skill.model.level as u8)
                    .and_then(|s| s.reuse_delay_group.as_ref())
                    .and_then(|v| v.text)
                    .unwrap_or(-1);

                inst.buffer.write_i32(if skill.passive { 1 } else { 0 })?;
                inst.buffer.write_u16(skill.model.level as u16)?;
                inst.buffer.write_u16(skill.model.sub_level as u16)?;
                inst.buffer.write_i32(skill.model.id)?;
                inst.buffer.write_i32(if reuse_group > 0 {
                    reuse_group
                } else {
                    skill.model.id
                })?;
                inst.buffer.write_u8(if skill.disabled { 1 } else { 0 })?;
                inst.buffer
                    .write_u8(if skill.can_enchant { 1 } else { 0 })?;
            }
        } else {
            inst.buffer.write_i32(0)?;
        }

        inst.buffer.write_i32(0)?; // lastLearnedSkillId

        Ok(inst)
    }
}

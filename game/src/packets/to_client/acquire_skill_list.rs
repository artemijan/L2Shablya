use l2_core::data::skill_tree_data::SkillTreesData;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

/// This packet is needed for letting the client know which skill he can learn
#[derive(Debug, Clone, SendablePacket)]
pub struct AcquireSkillList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl AcquireSkillList {
    pub const PACKET_ID: u8 = 0x90;
    pub fn new(p: &Player, skill_trees_data: &SkillTreesData) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;

        let learnable = skill_trees_data.get_available_skills(p);

        inst.buffer.write_u16(u16::try_from(learnable.len())?)?;
        for skill in learnable {
            inst.buffer.write_i32(skill.skill_id() as i32)?;
            inst.buffer.write_u16(u16::from(skill.skill_level()))?;
            inst.buffer.write_u64(skill.level_up_sp())?;
            inst.buffer.write_u8(skill.get_level())?;
            inst.buffer.write_u8(0)?; // Skill dual class level

            let items = skill.items();
            inst.buffer.write_u8(u8::try_from(items.len())?)?;
            for item in items {
                inst.buffer.write_i32(item.id() as i32)?;
                inst.buffer.write_u64(u64::from(item.count()))?;
            }

            let remove_skills = skill.remove_skills();
            inst.buffer.write_u8(u8::try_from(remove_skills.len())?)?;
            for rs in remove_skills {
                inst.buffer.write_i32(rs.skill_id() as i32)?;
                // In Java it writes the current level of the skill to be removed.
                // We need to find the current level of this skill in player's skills.
                let mut current_level = 0;
                if let Some(player_skills) = &p.skills {
                    for ps in player_skills {
                        if ps.model.id == rs.skill_id() as i32 {
                            current_level = ps.model.level;
                            break;
                        }
                    }
                }
                inst.buffer.write_u16(current_level as u16)?;
            }
        }
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use crate::packets::to_client::AcquireSkillList;
    use entities::entities::character;
    use l2_core::config::traits::ConfigDirLoader;
    use l2_core::data::char_template::ClassTemplates;
    use l2_core::data::skill_tree_data::SkillTreesData;
    use l2_core::game_objects::player::Player;

    #[tokio::test]
    async fn test_skill_list() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 1,
            face: 1,
            delete_at: None,
            user_id: 1,
            ..Default::default()
        };
        let templates = ClassTemplates::load();
        let skill_trees_data = SkillTreesData::load();
        let temp = templates.try_get_template(inst.class_id).unwrap().clone();
        let char = Player::new(inst, vec![], temp, None);
        let mut packet = AcquireSkillList::new(&char, &skill_trees_data).unwrap();
        // Packet starts with 0x90 and count. Since it's level 1, there might be some skills.
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(data[2], 0x90);
    }
}

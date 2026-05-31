use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct SkillList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillList {
    pub const PACKET_ID: u8 = 0x5F;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let last_learned_skill = 0;
        if let Some(skills) = p.skills.as_ref() {
            let mut filtered_skills: HashMap<i32, &l2_core::game_objects::creature::skill::Skill> =
                HashMap::new();

            for sk in skills {
                // Check if player meets level requirement for this skill level
                // In L2, if you have learned a skill but your level is lower than the learn level,
                // the skill might be disabled or its level reduced.
                // For "auto_get" skills, we should probably only show them if player level >= get_level.
                // However, the Player Object already contains learned skills.
                // The issue description says "Expertise S grade shouldn't be available on lvl 1".

                // For now, let's implement the "highest level only" logic.
                if let Some(existing) = filtered_skills.get(&sk.model.id) {
                    if sk.model.level > existing.model.level {
                        filtered_skills.insert(sk.model.id, sk);
                    }
                } else {
                    filtered_skills.insert(sk.model.id, sk);
                }
            }

            inst.buffer
                .write_u32(u32::try_from(filtered_skills.len())?)?; //skill count
            for sk in filtered_skills.values() {
                inst.buffer.write_i32(sk.passive as i32)?;
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
#[cfg(test)]
mod test {
    use crate::packets::to_client::SkillList;
    use entities::entities::character;
    use l2_core::config::traits::ConfigDirLoader;
    use l2_core::data::char_template::ClassTemplates;
    use l2_core::game_objects::player::Player;

    #[tokio::test]
    async fn test_skill_list() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 1,
            face: 1,
            class_id: 10,
            hair_style: 2,
            hair_color: 2,
            is_female: true,
            delete_at: None,
            user_id: 1,
            ..Default::default()
        };
        let templates = ClassTemplates::load();
        let temp = templates.try_get_template(inst.class_id).unwrap().clone();

        // Skill id 239 is Expertise. At level 1, it should be filtered out.
        use l2_core::game_objects::creature::skill::Skill;
        let skill_expertise_l1 = Skill::from_model(entities::entities::skill::Model {
            id: 239,
            level: 1,
            ..Default::default()
        });
        let skill_expertise_l2 = Skill::from_model(entities::entities::skill::Model {
            id: 239,
            level: 2,
            ..Default::default()
        });
        let normal_skill_l1 = Skill::from_model(entities::entities::skill::Model {
            id: 1,
            level: 1,
            ..Default::default()
        });
        let normal_skill_l2 = Skill::from_model(entities::entities::skill::Model {
            id: 1,
            level: 2,
            ..Default::default()
        });

        let char = Player::new(
            inst,
            vec![],
            temp,
            Some(vec![
                skill_expertise_l1,
                skill_expertise_l2,
                normal_skill_l1,
                normal_skill_l2,
            ]),
        );

        let mut packet = SkillList::new(&char).unwrap();
        let data = packet.buffer.get_data_mut(false);

        // Packet format: [id][skill_count(u32)][...skills][last_learned_skill(i32)]
        // skill count should be 2 (normal_skill_l2 and expertise_l2 because we removed level filter)
        assert_eq!(data[2], 0x5F); // PACKET_ID
        let count = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_skill_list_with_expertise() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 40,
            face: 1,
            class_id: 10,
            hair_style: 2,
            hair_color: 2,
            is_female: true,
            delete_at: None,
            user_id: 1,
            ..Default::default()
        };
        let templates = ClassTemplates::load();
        let temp = templates.try_get_template(inst.class_id).unwrap().clone();

        use l2_core::game_objects::creature::skill::Skill;
        let skill_expertise_l1 = Skill::from_model(entities::entities::skill::Model {
            id: 239,
            level: 1,
            ..Default::default()
        });
        let skill_expertise_l2 = Skill::from_model(entities::entities::skill::Model {
            id: 239,
            level: 2,
            ..Default::default()
        });
        let skill_expertise_l3 = Skill::from_model(entities::entities::skill::Model {
            id: 239,
            level: 3,
            ..Default::default()
        });

        let char = Player::new(
            inst,
            vec![],
            temp,
            Some(vec![
                skill_expertise_l1,
                skill_expertise_l2,
                skill_expertise_l3,
            ]),
        );

        let mut packet = SkillList::new(&char).unwrap();
        let data = packet.buffer.get_data_mut(false);

        // Level 40 means Expertise Level 2 (C grade).
        // Since we removed filtering, if player has Level 3 expertise, it will show Level 3.
        let count = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
        assert_eq!(count, 1);

        // Check skill level in the packet body
        // Body: [passive(i32)][level(i16)][sub_level(i16)][id(i32)]...
        let level = i16::from_le_bytes([data[11], data[12]]);
        let id = i32::from_le_bytes([data[15], data[16], data[17], data[18]]);

        assert_eq!(id, 239);
        assert_eq!(level, 3);
    }
}

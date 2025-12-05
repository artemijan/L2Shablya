use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct SkillList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillList {
    pub const PACKET_ID: u8 = 0x5F;
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
        if let Some(skills) = p.skills.as_ref() {
            inst.buffer.write_u32(u32::try_from(skills.len())?)?; //skill count
            for sk in skills {
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
        let char = Player::new(inst, vec![], temp);
        let mut packet = SkillList::new(&char).unwrap();
        assert_eq!(
            [95, 0, 0, 0, 0, 0, 0, 0, 0],
            packet.buffer.get_data_mut(false)[2..]
        );
    }
}

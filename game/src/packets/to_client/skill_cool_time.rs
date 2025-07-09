use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct SkillCoolTime {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillCoolTime {
    const PACKET_ID: u8 = 0xC7;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let len = player.skills.as_ref().map_or(0, Vec::len);
        inst.buffer.write_i32(i32::try_from(len)?)?;
        if let Some(skills) = player.skills.as_ref() {
            for _ in skills {
                //todo: implement me
            }
        }
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use crate::packets::to_client::SkillCoolTime;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::Player;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_skill_cool_time() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut m| {
            m.name = "Adelante".to_string();
            m.user_id = user.id;
            m
        })
        .await;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = GameController::from_config(cfg);
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let mut packet = SkillCoolTime::new(&player).unwrap();
        assert_eq!([199, 0, 0, 0, 0], packet.buffer.get_data_mut(false)[2..]);
    }
}

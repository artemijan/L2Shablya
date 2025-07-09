use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct QuestList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl QuestList {
    const PACKET_ID: u8 = 0x86;

    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(u16::try_from(p.quests.len())?)?;
        let mut onetime_quests = [0u8; 128]; //128 bytes
        for q in &p.quests {
            let q_id = q.get_id();
            if q.is_started() {
                inst.buffer.write_i32(q_id)?;
                inst.buffer.write_u32(q.get_condition_bit_set())?;
            } else if q.is_completed()
                && (q.is_completed() && !((q_id > 255 && q_id < 10256) || q_id > 11253))
            {
                onetime_quests[usize::try_from((q_id % 10000) / 8)?] |= 1 << (q_id % 8);
            }
        }
        inst.buffer.write_bytes(&onetime_quests)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    use crate::controller::GameController;
    use entities::entities::quest;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::quest::Quest;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use sea_orm::JsonValue;
    use test_utils::utils::get_test_db;
    #[tokio::test]
    async fn test_quest_list_packet() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let mut char = char_factory(&db_pool, |mut m| {
            m.name = "Adelante".to_string();
            m.user_id = user.id;
            m
        })
        .await;
        char.id = 268_476_204;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = GameController::from_config(cfg);
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let mut player = Player::new(char, vec![], template.clone());
        player.quests.push(Quest {
            model: quest::Model {
                char_id: player.char_model.id,
                name: "Tutorial".to_string(),
                quest_id: 255,
                variables: JsonValue::from_str(
                    r#"{"state":"started","condition":1, "memoState": 1}"#
                ).unwrap(),
            },
        });
        let p = QuestList::new(&player).unwrap();
        assert_eq!(
            [
                134, 1, 0, 255, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0
            ],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

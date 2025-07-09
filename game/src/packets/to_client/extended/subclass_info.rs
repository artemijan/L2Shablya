use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[repr(u8)]
pub enum SubclassInfoType {
    NoChanges = 0,
    NewSlotUsed = 1,
    ClassChanged = 2,
}

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct SubclassInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SubclassInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0xEA;
    pub fn new(p: &Player, the_type: SubclassInfoType) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write(the_type as u8)?;
        inst.buffer.write_i32(p.char_model.class_id)?;
        inst.buffer.write_i32(p.char_model.race_id)?;
        let subs = p.get_subclasses();
        inst.buffer.write_i32(i32::try_from(subs.len())?)?;
        for sub in subs {
            inst.buffer.write_i32(sub.index)?;
            inst.buffer.write_i32(sub.class_id)?;
            inst.buffer.write_i32(sub.level)?;
            inst.buffer.write(sub.class_type)?;
        }
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use crate::packets::to_client::extended::{SubclassInfo, SubclassInfoType};
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::Player;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_henna_ok() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut m| {
            m.name = "Adelante".to_string();
            m.user_id = user.id;
            m.class_id = 10;
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
        let p = SubclassInfo::new(&player, SubclassInfoType::NoChanges).unwrap();
        assert_eq!(
            [
                254, 234, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 1, 0,
                0, 0, 0
            ],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

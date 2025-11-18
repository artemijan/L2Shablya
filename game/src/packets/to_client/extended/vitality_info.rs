use l2_core::config::gs::GSServerConfig;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct VitalityInfo {
    pub buffer: SendablePacketBuffer,
    points: u32,
    vitality_bonus: u32,
    vitality_items_remaining: u32,
}

impl VitalityInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x118;


    pub fn new(player: &Player, config: &GSServerConfig) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            points: player.char_model.vitality_points,
            vitality_bonus: player.get_vitality_bonus(),
            vitality_items_remaining: config.vitality_max_items_allowed
                - player.get_vitality_used(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_u32(inst.points)?;
        inst.buffer.write_u32(inst.vitality_bonus)?;
        inst.buffer.write_u16(0u16)?; // Vitality additional bonus in %

        inst.buffer
            .write_u16(u16::try_from(inst.vitality_items_remaining)?)?;
        inst.buffer
            .write_u16(u16::try_from(config.vitality_max_items_allowed)?)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::Player;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use crate::packets::to_client::extended::VitalityInfo;

    #[tokio::test]
    async fn test_write_vitality_info() {
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
        let controller = GameController::from_config(cfg.clone()).await;
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let p = VitalityInfo::new(&player, &cfg).unwrap();
        assert_eq!(
            [254, 24, 1, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 231, 3, 231, 3],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}
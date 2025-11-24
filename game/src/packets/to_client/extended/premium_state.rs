use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct PremiumState {
    pub buffer: SendablePacketBuffer,
}

impl PremiumState {
    pub const PACKET_ID: u8 = 0xFE;
    pub const EX_PACKET_ID: u16 = 0xDA;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_u16(Self::EX_PACKET_ID)?;
        buffer.write_i32(p.get_object_id())?;
        buffer.write(p.has_premium())?;
        Ok(Self { buffer })
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
    use super::*;

    #[tokio::test]
    async fn test_premium_state() {
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
        let controller = GameController::from_config(cfg.clone()).await;
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let p = PremiumState::new(&player).unwrap();
        assert_eq!(
            [254, 218, 0, 44, 159, 0, 16, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

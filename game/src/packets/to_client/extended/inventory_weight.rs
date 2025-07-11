use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct InventoryWeight {
    pub(crate) buffer: SendablePacketBuffer,
}

impl InventoryWeight {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x166;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(p.char_model.id)?;
        inst.buffer.write_i32(p.inventory.get_current_load())?;
        inst.buffer.write_i32(p.inventory.get_max_load())?;
        Ok(inst)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    #[tokio::test]
    async fn test_inventory_weight() {
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
        let player = Player::new(char, vec![], template.clone());
        let p = InventoryWeight::new(&player).unwrap();
        assert_eq!(
            [254, 102, 1, 44, 159, 0, 16, 0, 0, 0, 0, 108, 24, 3, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

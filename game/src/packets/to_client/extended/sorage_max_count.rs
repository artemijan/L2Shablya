use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct StorageMaxCount {
    pub(crate) buffer: SendablePacketBuffer,
}

impl StorageMaxCount {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x2F;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_u32(p.inventory.get_limit())?;
        inst.buffer.write_u32(p.warehouse.get_limit())?;
        inst.buffer.write_u32(p.get_freight_slots())?;
        inst.buffer.write_u32(p.get_clan_warehouse_max_limit())?;
        inst.buffer.write_u32(p.get_private_store_sell_limit())?;
        inst.buffer.write_u32(p.get_private_store_buy_limit())?;
        inst.buffer.write_u32(p.get_dwarf_recipe_limit())?;
        inst.buffer.write_i32(p.get_common_recipe_limit())?;
        inst.buffer.write_i32(0)?; // todo: Belt inventory slots increase count
        inst.buffer.write_i32(p.get_quest_limit_count())?;
        inst.buffer.write_i32(40)?; // TODO: Find me!
        inst.buffer.write_i32(40)?; // TODO: Find me!
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
    async fn test_storage_max_count() {
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
        let controller = GameController::from_config(cfg).await;
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let p = StorageMaxCount::new(&player).unwrap();
        assert_eq!(
            [
                254, 47, 0, 80, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 200, 0, 0, 0, 3, 0, 0, 0, 4,
                0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 40, 0, 0, 0, 40, 0,
                0, 0
            ],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

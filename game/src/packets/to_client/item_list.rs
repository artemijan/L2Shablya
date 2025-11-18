use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use l2_core::traits::conversion::ToU32Rounded;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct ItemList {
    pub(crate) buffer: SendablePacketBuffer,
    show_window: bool,
}

impl ItemList {
    const PACKET_ID: u8 = 0x11;

    pub fn new(p: &Player, show_window: bool) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            show_window,
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(show_window)?;
        inst.buffer
            .write_u16(u16::try_from(p.inventory.items.len())?)?;
        for i in p.inventory.items.values() {
            let mask = i.calculate_mask();
            inst.buffer.write(u8::try_from(mask)?)?;
            inst.buffer.write_i32(i.item_model.id)?;
            inst.buffer.write_i32(i.get_display_id())?;
            if i.is_quest_item() {
                inst.buffer.write(1)?;
            } else if i.is_equipped() {
                inst.buffer.write(i.get_location())?;
            } else {
                inst.buffer.write(0xFF)?;
            }
            inst.buffer.write_i64(i.item_model.count)?;
            inst.buffer.write(i.get_type_2())?;
            inst.buffer.write(i.get_custom_type_1())?;
            inst.buffer.write(i.is_equipped())?;
            inst.buffer.write_i64(i.get_body_part())?;
            inst.buffer
                .write(u8::try_from(i.item_model.enchant_level)?)?; // Enchant level (pet level shown in control item)
            inst.buffer.write(i.get_custom_type_2())?; // Pet name exists or not shown in control item
            inst.buffer
                .write_u32(i.item_model.mana_left.to_u32_rounded()?)?;
            inst.buffer.write_i32(i.get_time())?;
            inst.buffer.write(i.is_available())?; // GOD Item enabled = 1 disabled (red) = 0
            //todo: implement me
        }
        if p.has_inventory_block() {
            //todo: implement me
        } else {
            inst.buffer.write_u16(0u16)?;
        }
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use crate::packets::to_client::ItemList;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::Player;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;

    #[tokio::test]
    async fn test_write_item_list() {
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
        let p = ItemList::new(&player, false).unwrap();
        assert_eq!(
            [17, 0, 0, 0, 0, 0, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

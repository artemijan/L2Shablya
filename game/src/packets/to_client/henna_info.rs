use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct HennaInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl HennaInfo {
    const PACKET_ID: u8 = 0xE5;

    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(0u16)?; // INT
        inst.buffer.write_u16(0u16)?; // STR
        inst.buffer.write_u16(0u16)?; // CON
        inst.buffer.write_u16(0u16)?; // MEN
        inst.buffer.write_u16(0u16)?; // DEX
        inst.buffer.write_u16(0u16)?; // WIT
        inst.buffer.write_u16(0u16)?; // LUC
        inst.buffer.write_u16(0u16)?; // CHA
        inst.buffer.write_u32(3 - p.get_henna_empty_slots())?; //slots
        inst.buffer.write_u32(0u32)?; //henna size
        //todo: implement me
        //for (Henna henna : _hennas)
        // 		{
        // 			buffer.writeInt(henna.getDyeId());
        // 			buffer.writeInt(henna.isAllowedClass(_player.getClassId()));
        // 		}
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye ID
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye Time Left
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye ID isValid
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use crate::packets::to_client::HennaInfo;
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
        let p = HennaInfo::new(&player).unwrap();
        assert_eq!(
            [
                229, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

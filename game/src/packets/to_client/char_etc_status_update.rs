use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Debug, Clone, SendablePacket)]
pub struct CharEtcStatusUpdate {
    pub buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CharEtcStatusUpdate {
    const PACKET_ID: u8 = 0xF9;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write(p.get_charges())?; // 1-7 increase force, level
        inst.buffer.write_i32(p.get_weight_penalty())?;
        inst.buffer.write(p.get_expertise_weapon_penalty())?;
        inst.buffer.write(p.get_expertise_armor_penalty())?;
        inst.buffer.write(0); // Death Penalty [1-15, 0 = disabled)], not used anymore in Ertheia
        inst.buffer.write(p.get_charged_souls())?;
        let mut mask: i32 = (p.block_all() || p.chat_banned() || p.silence_mode()).into();
        mask |= if p.is_in_instance_zone() { 2 } else { 0 };
        mask |= if p.has_charm_of_courage() { 4 } else { 0 };
        inst.buffer.write(u8::try_from(mask)?)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use crate::packets::to_client::CharEtcStatusUpdate;
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
        let p = CharEtcStatusUpdate::new(&player).unwrap();
        assert_eq!(
            [249, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Debug, Clone, SendablePacket)]
pub struct ShortcutsInit {
    pub(crate) buffer: SendablePacketBuffer,
}

impl ShortcutsInit {
    const PACKET_ID: u8 = 0x45;

    pub fn new(_p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u32(0u16)?; //count
        //todo: implement me
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
    use crate::packets::to_client::ShortcutsInit;

    #[tokio::test]
    async fn test_write_macro_list() {
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
        let p = ShortcutsInit::new(&player).unwrap();
        assert_eq!(
            [69, 0, 0, 0, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}

use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct FriendList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl FriendList {
    pub const PACKET_ID: u8 = 0x75;

    pub fn new(_player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let friends:Vec<u8> = vec![];
        inst.buffer.write_u32(u32::try_from(friends.len())?)?;
        //todo: implement me
        Ok(inst)
    }
}
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use test_utils::utils::get_test_db;
    use crate::controller::GameController;
    use super::*;
    
    #[tokio::test]
    async fn test_friend_list() {
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
        let packet = FriendList::new(&player).unwrap();
        assert_eq!(
            [117, 0, 0, 0, 0],
            packet.get_buffer().get_data_mut(false)[2..]
        );
    }
}
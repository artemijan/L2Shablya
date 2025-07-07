use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct BookmarkInfo {
    pub(crate) buffer: SendablePacketBuffer
}

impl BookmarkInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x85;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(0)?;
        inst.buffer.write_i32(player.get_bookmark_slot())?;
        let bookmarks = player.get_teleport_bookmarks();
        inst.buffer.write_i32(i32::try_from(bookmarks.len())?)?;
        for bookmark in bookmarks {
            inst.buffer.write_i32(bookmark.id)?;
            inst.buffer.write_i32(bookmark.x)?;
            inst.buffer.write_i32(bookmark.y)?;
            inst.buffer.write_i32(bookmark.z)?;
            inst.buffer.write_sized_c_utf16le_string(Some(&bookmark.name))?;
            inst.buffer.write_i32(bookmark.icon)?;
            inst.buffer.write_sized_c_utf16le_string(Some(&bookmark.tag))?;
        }
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
    use crate::packets::to_client::extended::BookmarkInfo;

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
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
        let controller = GameController::from_config(cfg);
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let p = BookmarkInfo::new(&player).unwrap();
        assert_eq!(
            [254, 133, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}
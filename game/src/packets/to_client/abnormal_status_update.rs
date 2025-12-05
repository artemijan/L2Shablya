use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;
use macro_common::SendablePacket;

#[derive(Debug, Clone, SendablePacket)]
pub struct AbnormalStatusUpdate {
    pub(crate) buffer: SendablePacketBuffer,
}

impl AbnormalStatusUpdate {
    pub const PACKET_ID: u8 = 0x85;

    pub fn new(_: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(0u16)?; // size
        //todo: implement me: buffs
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use entities::entities::character;
    use l2_core::config::traits::ConfigDirLoader;
    use l2_core::data::char_template::ClassTemplates;
    use l2_core::game_objects::player::Player;
    use crate::packets::to_client::AbnormalStatusUpdate;

    #[tokio::test]
    async fn test_skill_list() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 1,
            ..Default::default()
        };
        let templates = ClassTemplates::load();
        let temp = templates.try_get_template(inst.class_id).unwrap().clone();
        let char = Player::new(inst, vec![], temp);
        let mut packet = AbnormalStatusUpdate::new(&char).unwrap();
        assert_eq!(
            [133, 0, 0],
            packet.buffer.get_data_mut(false)[2..]
        );
    }
}

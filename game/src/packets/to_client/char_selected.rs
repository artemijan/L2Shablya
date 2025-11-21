use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct CharSelected {
    pub buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CharSelected {
    const PACKET_ID: u8 = 0x0B;
    const EX_PACKET_ID: Option<u16> = None;

    pub fn new(player: &Player, session_id: i32, game_time: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer
            .write_c_utf16le_string(Some(&player.char_model.name))?;
        inst.buffer.write_i32(player.char_model.id)?;
        inst.buffer
            .write_c_utf16le_string(player.char_model.title.as_deref())?;
        inst.buffer.write_i32(session_id)?;
        inst.buffer
            .write_i32(player.char_model.clan_id.unwrap_or(0))?;
        inst.buffer.write_i32(0)?; //???
        inst.buffer
            .write_i32(i32::from(player.char_model.is_female))?;
        inst.buffer
            .write_i32(i32::from(player.char_model.race_id))?;
        inst.buffer
            .write_i32(i32::from(player.char_model.class_id))?;
        inst.buffer.write_i32(1)?; //active?
        inst.buffer.write_i32(player.char_model.x)?;
        inst.buffer.write_i32(player.char_model.y)?;
        inst.buffer.write_i32(player.char_model.z)?;
        inst.buffer.write_f64(player.char_model.cur_hp)?;
        inst.buffer.write_f64(player.char_model.cur_mp)?;
        inst.buffer.write_i64(player.char_model.sp)?;
        inst.buffer.write_i64(player.char_model.exp)?;
        inst.buffer.write_i32(i32::from(player.char_model.level))?;
        inst.buffer.write_u32(player.char_model.reputation)?;
        inst.buffer.write_i32(player.char_model.pk_kills)?;
        inst.buffer.write_i32(game_time % (24 * 60))?;
        inst.buffer.write_i32(0)?;
        inst.buffer
            .write_i32(i32::from(player.char_model.class_id))?; // again? wtf
        inst.buffer.write_bytes(&[0; 16])?; //?
        inst.buffer.write_bytes(&[0; 36])?; //? 9 times x 4 bytes (i32)
        inst.buffer.write_bytes(&[0; 28])?; //?
        inst.buffer.write_i32(0)?; // last 4 bytes
        Ok(inst)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use entities::entities::character;
    use l2_core::config::traits::ConfigDirLoader;
    use l2_core::data::char_template::ClassTemplates;

    #[test]
    fn test_char_selected() {
        let inst = character::Model {
            name: "Adelante".to_string(),
            level: 1,
            face: 1,
            class_id: 10,
            hair_style: 2,
            hair_color: 2,
            is_female: true,
            delete_at: None,
            user_id: 1,
            cur_hp: 98.00,
            cur_mp: 59.00,
            online_time: Some(946),
            cur_cp: 49.00,
            x: -90939,
            y: 248_138,
            z: -3563,
            id: 268_476_204,
            ..Default::default()
        };
        let templates = ClassTemplates::load();
        let temp = templates.try_get_template(inst.class_id).unwrap().clone();
        let char = Player::new(inst, vec![], temp);
        let mut packet = CharSelected::new(&char, 9998, 286).unwrap();
        assert_eq!(
            [
                11, 65, 0, 100, 0, 101, 0, 108, 0, 97, 0, 110, 0, 116, 0, 101, 0, 0, 0, 44,
                159, 0, 16, 0, 0, 14, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 10,
                0, 0, 0, 1, 0, 0, 0, 197, 156, 254, 255, 74, 201, 3, 0, 21, 242, 255, 255, 0, 0, 0,
                0, 0, 128, 88, 64, 0, 0, 0, 0, 0, 128, 77, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 1, 0, 0, 0, 0, 0, 0, 10, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0
            ],
            packet.buffer.get_data_mut(false)[2..]
        );
    }
}

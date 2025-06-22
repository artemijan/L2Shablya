use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
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
        inst.buffer.write_i32(0)?; //todo clan_id
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
        inst.buffer
            .write_i32(i32::from(player.char_model.level))?;
        inst.buffer.write_i32(player.char_model.reputation)?;
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
    #[test]
    fn test_char_selected() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 1,
            face: 1,
            hair_style: 2,
            hair_color: 2,
            is_female: false,
            delete_at: None,
            user_id: 1,
            ..Default::default()
        };
        let char = Player::new(inst, vec![]);
        let mut packet = CharSelected::new(&char, 1, 0).unwrap();
        assert_eq!(
            [
                199, 0, 11, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0
            ],
            packet.buffer.get_data_mut(false)
        );
    }
}

use entities::dao::char_info::CharacterInfo;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct CharSelected {
    buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CharSelected {
    const PACKET_ID: u8 = 0x0B;
    const EX_PACKET_ID: Option<u16> = None;

    pub fn new(char_info: &CharacterInfo, session_id: i32, game_time: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_string(Some(&char_info.char_model.name))?;
        inst.buffer.write_i32(char_info.char_model.id)?;
        inst.buffer
            .write_string(char_info.char_model.title.as_deref())?;
        inst.buffer.write_i32(session_id)?;
        inst.buffer.write_i32(0)?; //todo clan_id
        inst.buffer.write_i32(0)?; //???
        inst.buffer
            .write_i32(i32::from(char_info.char_model.is_female))?;
        inst.buffer
            .write_i32(i32::from(char_info.char_model.race_id))?;
        inst.buffer
            .write_i32(i32::from(char_info.char_model.class_id))?;
        inst.buffer.write_i32(1)?; //active?
        inst.buffer.write_i32(char_info.char_model.x)?;
        inst.buffer.write_i32(char_info.char_model.y)?;
        inst.buffer.write_i32(char_info.char_model.z)?;
        inst.buffer.write_f64(char_info.char_model.cur_hp)?;
        inst.buffer.write_f64(char_info.char_model.cur_mp)?;
        inst.buffer.write_i64(char_info.char_model.sp)?;
        inst.buffer.write_i64(char_info.char_model.exp)?;
        inst.buffer
            .write_i32(i32::from(char_info.char_model.level))?;
        inst.buffer.write_i32(char_info.char_model.reputation)?;
        inst.buffer.write_i32(char_info.char_model.pk_kills)?;
        inst.buffer.write_i32(game_time % (24 * 60))?;
        inst.buffer.write_i32(0)?;
        inst.buffer
            .write_i32(i32::from(char_info.char_model.class_id))?; // again? wtf
        inst.buffer.write_bytes(&[0; 16])?; //?
        inst.buffer.write_bytes(&[0; 36])?; //? 9 times x 4 bytes (i32)
        inst.buffer.write_bytes(&[0; 28])?; //?
        inst.buffer.write_i32(0)?; // last 4 bytes
        Ok(inst)
    }
}

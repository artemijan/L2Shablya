use async_trait::async_trait;
use entities::entities::character;
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::packets::common::SendablePacket;
use l2_core::packets::write::SendablePacketBuffer;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct CharSelectionInfo {
    buffer: SendablePacketBuffer,
    session_id: i32,
    active_id: i32,
}

impl CharSelectionInfo {
    const PACKET_ID: u8 = 0x09;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    pub async fn new(
        account_name: &str,
        session_id: i32,
        cfg: &GSServer,
        db_pool: &mut DBPool,
    ) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        let chars = character::Model::find_characters_by_username(db_pool, account_name).await?;
        let char_len = chars.len() as u32;
        buffer.write_u32(char_len)?;
        buffer.write_u32(u32::from(cfg.max_chars_on_account))?;
        buffer.write_bool(char_len == u32::from(cfg.max_chars_on_account))?;
        buffer.write(1)?; // 0=can't play, 1=can play free until level 85, 2=100% free play
        buffer.write_u32(2)?; // if 1, Korean client
        buffer.write(0)?; // Balthus Knights, if 1 suggests premium account
        let mut last_access = None;
        let mut active_id = -1;
        for (index, char) in chars.iter().enumerate() {
            if char.last_access > last_access {
                last_access = char.last_access;
                active_id = index as i32;
            }
            buffer.write_string(Some(&char.name))?;
            buffer.write_i32(char.id)?;
            buffer.write_string(Some(account_name))?;
            buffer.write_i32(session_id)?;
            buffer.write_i32(0)?; // clan id
            buffer.write_i32(0)?; // Builder level
            buffer.write_i32(i32::from(char.sex))?;
            buffer.write_i32(i32::from(char.race_id))?;
            buffer.write_i32(i32::from(char.base_class_id))?;
            buffer.write_i32(1)?; // GameServerName
            buffer.write_i32(char.x)?;
            buffer.write_i32(char.y)?;
            buffer.write_i32(char.z)?;
            buffer.write_f64(char.cur_hp)?;
            buffer.write_f64(char.cur_mp)?;
            buffer.write_i64(char.sp)?;
            buffer.write_i64(char.exp)?;
            todo!()
        }
        Ok(Self {
            buffer,
            session_id,
            active_id,
        })
    }
}

#[async_trait]
impl SendablePacket for CharSelectionInfo {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

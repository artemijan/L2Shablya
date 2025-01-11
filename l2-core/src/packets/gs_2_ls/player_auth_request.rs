use crate::packets::common::{ReadablePacket, SendablePacket};
use crate::packets::read::ReadablePacketBuffer;
use crate::packets::write::SendablePacketBuffer;
use crate::session::SessionKey;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerAuthRequest {
    pub session: SessionKey,
    pub account_name: String,
    buffer: SendablePacketBuffer,
}
impl PlayerAuthRequest {
    ///
    /// # Errors
    /// - packet size is too large
    pub fn new(account_name: &str, session: SessionKey) -> anyhow::Result<Self> {
        let mut inst = Self {
            account_name: account_name.to_string(),
            session,
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(0x05)?;
        inst.buffer.write_string(Some(&inst.account_name))?;
        inst.buffer.write_i32(inst.session.play_ok1)?;
        inst.buffer.write_i32(inst.session.play_ok2)?;
        inst.buffer.write_i32(inst.session.login_ok1)?;
        inst.buffer.write_i32(inst.session.login_ok2)?;
        Ok(inst)
    }
}
impl ReadablePacket for PlayerAuthRequest {
    const PACKET_ID: u8 = 0x05;

    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        let play_ok1 = buffer.read_i32();
        let play_ok2 = buffer.read_i32();
        let login_ok1 = buffer.read_i32();
        let login_ok2 = buffer.read_i32();
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
            session: SessionKey {
                play_ok1,
                play_ok2,
                login_ok1,
                login_ok2,
            },
        })
    }
}
#[async_trait]
impl SendablePacket for PlayerAuthRequest {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

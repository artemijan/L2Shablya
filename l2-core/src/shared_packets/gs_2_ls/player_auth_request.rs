use macro_common::SendablePacketImpl;
use crate::session::SessionKey;
use crate::shared_packets::common::{ReadablePacket};
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use crate as l2_core;

#[derive(Clone, Debug, SendablePacketImpl)]
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
        inst.buffer.write_c_utf16le_string(Some(&inst.account_name))?;
        inst.buffer.write_i32(inst.session.play_ok1)?;
        inst.buffer.write_i32(inst.session.play_ok2)?;
        inst.buffer.write_i32(inst.session.login_ok1)?;
        inst.buffer.write_i32(inst.session.login_ok2)?;
        Ok(inst)
    }
}
impl ReadablePacket for PlayerAuthRequest {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_string()?;
        let play_ok1 = buffer.read_i32()?;
        let play_ok2 = buffer.read_i32()?;
        let login_ok1 = buffer.read_i32()?;
        let login_ok2 = buffer.read_i32()?;
        Ok(Self {
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


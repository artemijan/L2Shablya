use crate::shared_packets::{
    common::{ReadablePacket},
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};
use macro_common::SendablePacketImpl;
use crate as l2_core;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct PlayerAuthResponse {
    pub buffer: SendablePacketBuffer,
    pub account: String,
    pub is_ok: bool,
}

impl PlayerAuthResponse {
    #[must_use]
    pub fn new(account: &str, is_ok: bool) -> PlayerAuthResponse {
        let mut gg = PlayerAuthResponse {
            buffer: SendablePacketBuffer::new(),
            account: account.to_string(),
            is_ok,
        };
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x03)?;
        self.buffer.write_c_utf16le_string(Some(&self.account))?;
        self.buffer.write_u8(u8::from(self.is_ok))?;
        Ok(())
    }
}

impl ReadablePacket for PlayerAuthResponse {
    const PACKET_ID: u8 = 0x03;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let _packet_id = buffer.read_byte()?;
        let account = buffer.read_string()?;
        let is_ok = buffer.read_boolean()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            is_ok,
            account,
        })
    }
}

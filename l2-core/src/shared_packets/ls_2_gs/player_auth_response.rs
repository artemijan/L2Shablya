use crate::shared_packets::{
    common::{ReadablePacket, SendablePacket},
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};

#[derive(Debug, Clone)]
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
        self.buffer.write_string(Some(&self.account))?;
        self.buffer.write_u8(u8::from(self.is_ok))?;
        Ok(())
    }
}

impl SendablePacket for PlayerAuthResponse {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

impl ReadablePacket for PlayerAuthResponse {
    const PACKET_ID: u8 = 0x03;

    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        let _packet_id = buffer.read_byte();
        let account = buffer.read_string();
        let is_ok = buffer.read_boolean();
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            is_ok,
            account,
        })
    }
}

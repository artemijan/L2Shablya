use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct RestartResponse {
    pub buffer: SendablePacketBuffer,
}

impl RestartResponse {
    pub const PACKET_ID: u8 = 0x71;
    pub fn new(ok: bool) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_i32(ok)?;
        Ok(Self { buffer })
    }
    pub fn Ok() -> anyhow::Result<Self> {
        Self::new(true)
    }
    pub fn NotOk() -> anyhow::Result<Self> {
        Self::new(false)
    }
}

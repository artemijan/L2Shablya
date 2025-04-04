use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
#[allow(unused)]
pub struct PlayerLoginResponse {
    buffer: SendablePacketBuffer,
    reason: u32,
    is_ok: bool,
}

impl PlayerLoginResponse {
    pub const PACKET_ID: u8 = 0x0A;
    pub const SYSTEM_ERROR_LOGIN_LATER: u32 = 1;
    pub fn ok() -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_i32(-1)?;
        buffer.write_u32(0u32)?;
        Ok(Self {
            buffer,
            reason: 0,
            is_ok: true,
        })
    }
    pub fn fail(reason: u32) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_i32(0)?;
        buffer.write_u32(reason)?;
        Ok(Self {
            buffer,
            reason,
            is_ok: false,
        })
    }
}

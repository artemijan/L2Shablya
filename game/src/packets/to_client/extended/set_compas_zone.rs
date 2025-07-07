use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct SetCompasZoneCode {
    pub buffer: SendablePacketBuffer,
}

impl SetCompasZoneCode {
    pub const PACKET_ID: u8 = 0x2F;
    pub const EX_PACKET_ID: u16 = 0x33;
    pub fn new(compas_zone:i32) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_u16(Self::EX_PACKET_ID)?;
        buffer.write_i32(compas_zone)?;
        Ok(Self { buffer })
    }
}

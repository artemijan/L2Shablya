use l2_core::game_objects::player::Player;
use l2_core::game_objects::zone::Location;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct MoveTo {
    pub buffer: SendablePacketBuffer,
}

impl MoveTo {
    pub const PACKET_ID: u8 = 0x2F;
    pub fn new(p: &Player, loc: &Location) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_i32(p.char_model.id)?;
        buffer.write_i32(p.location.x)?;
        buffer.write_i32(p.location.y)?;
        buffer.write_i32(p.location.z)?;
        buffer.write_i32(loc.x)?;
        buffer.write_i32(loc.y)?;
        buffer.write_i32(loc.z)?;
        Ok(Self { buffer })
    }
}

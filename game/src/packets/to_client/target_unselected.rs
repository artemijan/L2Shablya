use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct TargetUnselected {
    pub buffer: SendablePacketBuffer,
}

impl TargetUnselected {
    pub const PACKET_ID: u8 = 0x24;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_i32(p.get_object_id())?;
        let p_loc = p.get_location();
        buffer.write_i32(p_loc.x)?;
        buffer.write_i32(p_loc.y)?;
        buffer.write_i32(p_loc.z)?;
        buffer.write_i32(0)?; //todo: what is this?
        Ok(Self { buffer })
    }
}

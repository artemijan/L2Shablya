use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;


#[allow(unused)]
#[derive(Clone, Debug, SendablePacket)]
pub struct CharMoveToLocation {
    pub buffer: SendablePacketBuffer
}

#[allow(unused)]
impl CharMoveToLocation {
    const PACKET_ID: u8 = 0x2F;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new(p: &Player, target_x: i32, target_y: i32, target_z: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(p.get_id())?; // 1-7 increase force, level

        // Target position
        inst.buffer.write_i32(target_x)?;
        inst.buffer.write_i32(target_y)?;
        inst.buffer.write_i32(target_z)?;

        // Source position
        inst.buffer.write_i32(p.get_x())?;
        inst.buffer.write_i32(p.get_y())?;
        inst.buffer.write_i32(p.get_z())?;

        Ok(inst)
    }
}

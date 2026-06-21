use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Clone, Debug, SendablePacket)]
pub struct Attack {
    pub buffer: SendablePacketBuffer,
}

impl Attack {
    pub const PACKET_ID: u8 = 0x33;

    pub fn new(
        attacker_id: i32,
        target_id: i32,
        damage: i32,
        flags: i32,
        x: i32,
        y: i32,
        z: i32,
        target_x: i32,
        target_y: i32,
        target_z: i32,
    ) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(attacker_id)?;
        inst.buffer.write_i32(target_id)?;
        inst.buffer.write_i32(0)?; // soulshot visual substitute
        inst.buffer.write_i32(damage)?;
        inst.buffer.write_i32(flags)?;
        inst.buffer.write_i32(0)?; // grade
        inst.buffer.write_i32(x)?;
        inst.buffer.write_i32(y)?;
        inst.buffer.write_i32(z)?;
        inst.buffer.write_u16(0u16)?; // Hits count - 1
        inst.buffer.write_i32(target_x)?;
        inst.buffer.write_i32(target_y)?;
        inst.buffer.write_i32(target_z)?;

        Ok(inst)
    }
}

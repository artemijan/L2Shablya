use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Clone, Debug, SendablePacket)]
pub struct MagicSkillUse {
    pub buffer: SendablePacketBuffer,
}

impl MagicSkillUse {
    pub const PACKET_ID: u8 = 0x48;

    pub fn new(
        char_id: i32,
        target_id: i32,
        skill_id: i32,
        skill_level: i32,
        hit_time: i32,
        reuse_delay: i32,
        reuse_group: i32,
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
        inst.buffer.write_i32(1)?; // Casting bar type: 0 - default, 1 - default up, 2 - blue, 3 - green, 4 - red.
        inst.buffer.write_i32(char_id)?;
        inst.buffer.write_i32(target_id)?;
        inst.buffer.write_i32(skill_id)?;
        inst.buffer.write_i32(skill_level)?;
        inst.buffer.write_i32(hit_time)?;
        inst.buffer.write_i32(reuse_group)?; // reuse group
        inst.buffer.write_i32(reuse_delay)?;
        inst.buffer.write_i32(x)?;
        inst.buffer.write_i32(y)?;
        inst.buffer.write_i32(z)?;
        inst.buffer.write_u16(0u16)?; // isGroundTargetSkill ? 65535 : 0
        inst.buffer.write_u16(0u16)?; // has ground location
        inst.buffer.write_i32(target_x)?;
        inst.buffer.write_i32(target_y)?;
        inst.buffer.write_i32(target_z)?;
        inst.buffer.write_i32(0)?; // isActionIdUsed
        inst.buffer.write_i32(0)?; // actionId

        Ok(inst)
    }
}

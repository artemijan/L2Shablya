use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Clone, Debug, SendablePacket)]
pub struct MagicSkillLaunched {
    pub buffer: SendablePacketBuffer,
}

impl MagicSkillLaunched {
    pub const PACKET_ID: u8 = 0x54;

    pub fn new(
        char_id: i32,
        skill_id: i32,
        skill_level: i32,
        casting_type: i32,
        targets: &[i32],
    ) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(casting_type)?;
        inst.buffer.write_i32(char_id)?;
        inst.buffer.write_i32(skill_id)?;
        inst.buffer.write_i32(skill_level)?;
        inst.buffer.write_i32(targets.len() as i32)?;
        for &target_id in targets {
            inst.buffer.write_i32(target_id)?;
        }

        Ok(inst)
    }
}

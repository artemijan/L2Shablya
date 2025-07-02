use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct QuestList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl QuestList {
    const PACKET_ID: u8 = 0x86;

    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(u16::try_from(p.quests.len())?)?;
        let mut onetime_quests = [0u8; 1024]; //128 bytes
        for q in &p.quests {
            let q_id = q.get_id();
            if q.get_id() > 0
                && !q.is_started()
                && (q.is_completed() && !((q_id > 255 && q_id < 10256) || q_id > 11253))
            {
                onetime_quests[usize::try_from((q_id % 10000) / 8)?] |= 1 << (q_id % 8);
            }
            if q.is_started() {
                inst.buffer.write_i32(q_id)?;
                inst.buffer.write_i32(q.get_condition_bit_set())?;
            }
        }
        inst.buffer.write_bytes(&onetime_quests)?;
        Ok(inst)
    }
}

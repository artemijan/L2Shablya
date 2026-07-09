use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct SkillCoolTime {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillCoolTime {
    pub const PACKET_ID: u8 = 0xC7;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut active_reuses = Vec::new();
        for reuse in &player.skill_reused {
            if reuse.has_not_passed() {
                active_reuses.push(reuse);
            }
        }
        Self::create_packet(&active_reuses)
    }

    pub fn for_skill(player: &Player, skill_id: i32) -> anyhow::Result<Self> {
        let mut active_reuses = Vec::new();
        let mut reuse_group = -1;

        for reuse in &player.skill_reused {
            if reuse.skill_id == skill_id && reuse.has_not_passed() {
                reuse_group = reuse.shared_reuse_group;
                break;
            }
        }

        for reuse in &player.skill_reused {
            if reuse.has_not_passed()
                && (reuse.skill_id == skill_id
                    || (reuse_group > 0 && reuse.shared_reuse_group == reuse_group))
            {
                active_reuses.push(reuse);
            }
        }

        Self::create_packet(&active_reuses)
    }

    fn create_packet(
        active_reuses: &[&l2_core::game_objects::creature::skill::SkillReuse],
    ) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };

        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(active_reuses.len() as i32)?;
        for reuse in active_reuses {
            let reuse_ms = reuse.reuse_delay;
            let remaining_ms = reuse.get_remaining();
            inst.buffer.write_i32(if reuse.shared_reuse_group > 0 {
                reuse.shared_reuse_group
            } else {
                reuse.skill_id
            })?;
            inst.buffer.write_i32(reuse.skill_level)?;
            inst.buffer
                .write_i32((if reuse_ms > 0 { reuse_ms } else { remaining_ms } / 1000) as i32)?;
            inst.buffer.write_i32((remaining_ms / 1000) as i32)?;
        }

        Ok(inst)
    }
}
#[cfg(test)]
mod test {}

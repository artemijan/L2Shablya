use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SkillCoolTime {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SkillCoolTime {
    const PACKET_ID: u8 = 0xC7;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let len = player.skills.as_ref().map_or(0, Vec::len);
        inst.buffer.write_i32(i32::try_from(len)?)?;
        if let Some(skills) = player.skills.as_ref() {
            for _ in skills {
                //todo: implement me
            }
        }
        Ok(inst)
    }
}

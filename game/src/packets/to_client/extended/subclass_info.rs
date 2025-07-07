use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[repr(u8)]
pub enum SubclassInfoType {
    NoChanges,
    NewSlotUsed,
    ClassChanged,
}

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct SubclassInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl SubclassInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0xEA;
    pub fn new(p: &Player, the_type: SubclassInfoType) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write(the_type as u8)?;
        inst.buffer.write_i32(p.char_model.class_id)?; //todo: should be id from the template
        inst.buffer.write_i32(p.char_model.race_id)?; //todo: should be id from the template
        let subs = p.get_subclasses();
        inst.buffer.write_i32(i32::try_from(subs.len())?)?;
        for sub in subs {
            inst.buffer.write_i32(sub.index)?;
            inst.buffer.write_i32(sub.class_id)?;
            inst.buffer.write_i32(sub.level)?;
            inst.buffer.write(sub.class_type)?;
        }
        Ok(inst)
    }
}

use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct HennaInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl HennaInfo {
    const PACKET_ID: u8 = 0xE5;

    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(0u16)?; // INT
        inst.buffer.write_u16(0u16)?; // STR
        inst.buffer.write_u16(0u16)?; // CON
        inst.buffer.write_u16(0u16)?; // MEN
        inst.buffer.write_u16(0u16)?; // DEX
        inst.buffer.write_u16(0u16)?; // WIT
        inst.buffer.write_u16(0u16)?; // LUC
        inst.buffer.write_u16(0u16)?; // CHA
        inst.buffer.write_u32(3 - p.get_henna_slots())?; //slots 
        inst.buffer.write_u32(0u32)?; //henna size
        //todo: implement me
        //for (Henna henna : _hennas)
        // 		{
        // 			buffer.writeInt(henna.getDyeId());
        // 			buffer.writeInt(henna.isAllowedClass(_player.getClassId()));
        // 		}
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye ID
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye Time Left
        inst.buffer.write_u32(0u32)?; // Premium Slot Dye ID isValid
        Ok(inst)
    }
}

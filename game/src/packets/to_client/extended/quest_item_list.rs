use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct QuestItemList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl QuestItemList {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0xC7;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_u16(0u16)?; //quest items count
        //todo: implement me
        if player.has_inventory_block(){
            //todo: implement me
        }else{
            inst.buffer.write_u16(0u16)?;
        }
        Ok(inst)
    }
}

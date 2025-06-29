use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct InventoryAdenaInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl InventoryAdenaInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x13E;
    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_u64(p.inventory.get_adena())?;
        inst.buffer.write_u16(p.inventory.get_size())?;
        Ok(inst)
    }
}

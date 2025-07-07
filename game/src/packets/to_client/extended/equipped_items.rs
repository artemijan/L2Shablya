use l2_core::bitmask::BitMask;
use l2_core::game_objects::item::ItemObject;
use l2_core::game_objects::player::inventory::InventorySlot;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use sea_orm::Iterable;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct EquippedItems {
    pub(crate) buffer: SendablePacketBuffer,
}

impl EquippedItems {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x156;
    pub fn new(p: &Player, with_all_slots: bool) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(p.char_model.id)?;
        let mut bitmask = BitMask::new(40); //5 bytes, 50 bits
        let slots = InventorySlot::iter();
        let num_slots = slots.len();
        if with_all_slots {
            for slot in slots {
                bitmask.add_mask(slot as u32);
            }
        }
        inst.buffer.write_u16(u16::try_from(num_slots)?)?;
        inst.buffer.write_bytes(bitmask.flags())?;
        for slot in InventorySlot::iter() {
            if bitmask.contains_mask(slot) {
                let item = p.get_item_by_slot(slot);
                inst.buffer.write_u16(22u16)?; // 10 + 4 * 3
                inst.buffer.write_i32(item.map_or(0, |i| i.item_model.id))?;
                inst.buffer
                    .write_i32(item.map_or(0, |i| i.item_model.item_id))?;
                let augmentation = item
                    .and_then(|i| i.item_model.get_augmentation())
                    .unwrap_or((0, 0, 0));
                inst.buffer.write_i32(augmentation.0)?;
                inst.buffer.write_i32(augmentation.1)?;
                inst.buffer
                    .write_i32(item.map_or(0, ItemObject::get_visual_id))?;
            }
        }
        Ok(inst)
    }
}

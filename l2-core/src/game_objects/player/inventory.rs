use std::collections::HashMap;
use crate::game_objects::item::ItemObject;
use crate::game_objects::player::paper_doll::PaperDoll;
use entities::entities::item;
use sea_orm::EnumIter;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub items: HashMap<i32,ItemObject>,
}

impl Inventory {
    pub fn from_items(items: Vec<item::Model>) -> Self {
        Self {
            items: ItemObject::from_items(items),
        }
    }
    #[must_use]
    pub fn get_talisman_slots(&self) -> u8 {
        //todo: implement me
        6
    }
    #[must_use]
    pub fn get_brooch_jewel_slots(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_limit(&self) -> u8 {
        //todo: implement me
        80
    }
    #[must_use]
    pub fn get_current_load(&self) -> i32 {
        0i32 //todo: implement me
    }
    #[must_use]
    pub fn get_max_load(&self) -> i32 {
        202_860 //todo: implement me
    }
    #[must_use]
    pub fn get_adena(&self) -> u64 {
        0u64 //todo: implement me
    }
    #[must_use]
    pub fn get_size(&self) -> u16 {
        0u16 //todo: implement me
    }
}

#[repr(u8)]
#[derive(Debug, EnumIter, Clone, Copy)]
pub enum InventorySlot {
    Under,
    Rear,
    Lear,
    Neck,
    RFinger,
    LFinger,
    Head,
    RHand,
    LHand,
    Gloves,
    Chest,
    Legs,
    Feet,
    Cloak,
    LrHand,
    Hair,
    Hair2,
    RBracelet,
    LBracelet,
    Deco1,
    Deco2,
    Deco3,
    Deco4,
    Deco5,
    Deco6,
    Belt,
    Brooch,
    BroochJewel,
    BroochJewel2,
    BroochJewel3,
    BroochJewel4,
    BroochJewel5,
    BroochJewel6,
}

impl From<InventorySlot> for u32 {
    fn from(value: InventorySlot) -> Self {
        value as u32
    }
}
impl From<InventorySlot> for PaperDoll {
    fn from(slot: InventorySlot) -> Self {
        match slot {
            InventorySlot::Under => PaperDoll::Under,
            InventorySlot::Rear => PaperDoll::Rear,
            InventorySlot::Lear => PaperDoll::Lear,
            InventorySlot::Neck => PaperDoll::Neck,
            InventorySlot::RFinger => PaperDoll::RFinger,
            InventorySlot::LFinger => PaperDoll::LFinger,
            InventorySlot::Head => PaperDoll::Head,
            InventorySlot::LHand => PaperDoll::LHand,
            InventorySlot::Gloves => PaperDoll::Gloves,
            InventorySlot::Chest => PaperDoll::Chest,
            InventorySlot::Legs => PaperDoll::Legs,
            InventorySlot::Feet => PaperDoll::Feet,
            InventorySlot::Cloak => PaperDoll::Cloak,
            InventorySlot::LrHand | InventorySlot::RHand => PaperDoll::RHand,
            InventorySlot::Hair => PaperDoll::Hair,
            InventorySlot::Hair2 => PaperDoll::Hair2,
            InventorySlot::RBracelet => PaperDoll::RBracelet,
            InventorySlot::LBracelet => PaperDoll::LBracelet,
            InventorySlot::Deco1 => PaperDoll::Deco1,
            InventorySlot::Deco2 => PaperDoll::Deco2,
            InventorySlot::Deco3 => PaperDoll::Deco3,
            InventorySlot::Deco4 => PaperDoll::Deco4,
            InventorySlot::Deco5 => PaperDoll::Deco5,
            InventorySlot::Deco6 => PaperDoll::Deco6,
            InventorySlot::Belt => PaperDoll::Belt,
            InventorySlot::Brooch => PaperDoll::Brooch,
            InventorySlot::BroochJewel => PaperDoll::BroochJewel1,
            InventorySlot::BroochJewel2 => PaperDoll::BroochJewel2,
            InventorySlot::BroochJewel3 => PaperDoll::BroochJewel3,
            InventorySlot::BroochJewel4 => PaperDoll::BroochJewel4,
            InventorySlot::BroochJewel5 => PaperDoll::BroochJewel5,
            InventorySlot::BroochJewel6 => PaperDoll::BroochJewel6,
        }
    }
}

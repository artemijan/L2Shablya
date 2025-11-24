use entities::dao::item::{ItemVariables, LocType};
use entities::entities::item;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperDoll {
    Under = 0,
    Head = 1,
    Hair = 2,
    Hair2 = 3,
    Neck = 4,
    RHand = 5,
    Chest = 6,
    LHand = 7,
    Rear = 8,
    Lear = 9,
    Gloves = 10,
    Legs = 11,
    Feet = 12,
    RFinger = 13,
    LFinger = 14,
    LBracelet = 15,
    RBracelet = 16,
    Deco1 = 17,
    Deco2 = 18,
    Deco3 = 19,
    Deco4 = 20,
    Deco5 = 21,
    Deco6 = 22,
    Cloak = 23,
    Belt = 24,
    Brooch = 25,
    BroochJewel1 = 26,
    BroochJewel2 = 27,
    BroochJewel3 = 28,
    BroochJewel4 = 29,
    BroochJewel5 = 30,
    BroochJewel6 = 31,
    TotalSlots = 32,
}

impl PaperDoll {
    #[must_use]
    pub fn ordered_ids() -> [PaperDoll; 33] {
        [
            Self::Under,
            Self::Rear,
            Self::Lear,
            Self::Neck,
            Self::RFinger,
            Self::LFinger,
            Self::Head,
            Self::RHand,
            Self::LHand,
            Self::Gloves,
            Self::Chest,
            Self::Legs,
            Self::Feet,
            Self::Cloak,
            Self::RHand, // I don't give a fuck why Rhand declared twice, copied as is from L2j
            Self::Hair,
            Self::Hair2,
            Self::RBracelet,
            Self::LBracelet,
            Self::Deco1,
            Self::Deco2,
            Self::Deco3,
            Self::Deco4,
            Self::Deco5,
            Self::Deco6,
            Self::Belt,
            Self::Brooch,
            Self::BroochJewel1,
            Self::BroochJewel2,
            Self::BroochJewel3,
            Self::BroochJewel4,
            Self::BroochJewel5,
            Self::BroochJewel6,
        ]
    }

    #[must_use]
    pub fn visual_ids() -> [PaperDoll; 9] {
        [
            Self::RHand,
            Self::LHand,
            Self::Gloves,
            Self::Chest,
            Self::Legs,
            Self::Feet,
            Self::RHand, // I don't give a fuck why Rhand declared twice, copied as is from L2j
            Self::Hair,
            Self::Hair2,
        ]
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn restore_visible_inventory(items: &std::collections::HashMap<i32, crate::game_objects::item::ItemObject>) -> [[i32; 4]; 33] {
        let mut result = [[0; 4]; 33];
        for item in items.values() {
            if item.item_model.loc == LocType::Paperdoll {
                let slot = item.item_model.loc_data;
                result[slot as usize][0] = item.object_id;
                result[slot as usize][1] = item.item_model.item_id;
                result[slot as usize][2] = item.item_model.enchant_level;
                result[slot as usize][3] = item
                    .item_model
                    .variables
                    .get(ItemVariables::VisualId.as_key())
                    .and_then(serde_json::Value::as_i64)
                    .and_then(|v| i32::try_from(v).ok())
                    .unwrap_or(0);
                if result[slot as usize][3] > 0 {
                    // fix for hair appearance conflicting with original model
                    result[slot as usize][1] = result[slot as usize][3];
                }
            }
        }
        result
    }
}

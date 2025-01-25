use crate::entities::{character, item};
use chrono::Utc;
use sea_orm::DbErr;
use serde_json::Value;

#[repr(i32)]
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
}
#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub char_model: character::Model,
    pub items: Vec<item::Model>,
    pub paperdoll: [[i32; 4]; 33],
}

#[allow(clippy::missing_errors_doc)]
impl CharacterInfo {
    const VITALITY_ITEMS_USED_VARIABLE_NAME: &'static str = "VITALITY_ITEMS_USED";
    const HAIR_ACCESSORY_VARIABLE_NAME: &'static str = "HAIR_ACCESSORY_ENABLED";
    pub fn new(char_model: character::Model, items: Vec<item::Model>) -> Result<Self, DbErr> {
        let paperdoll = char_model.restore_visible_inventory(&items)?;
        Ok(Self {
            char_model,
            items,
            paperdoll,
        })
    }
    #[must_use]
    pub fn paper_doll_item_id(&self, slot: PaperDoll) -> i32 {
        self.paperdoll[slot as usize][1]
    }

    #[must_use]
    pub fn get_item(&self, item_id: i32) -> Option<&item::Model> {
        self.items.iter().find(|i| i.item_id == item_id)
    }

    #[must_use]
    pub fn get_weapon(&self) -> Option<&item::Model> {
        let r_hand = self.get_paper_doll_object_id(PaperDoll::RHand);
        if r_hand > 0 {
            return self.get_item(r_hand);
        }
        None
    }

    #[must_use]
    pub fn get_paper_doll_object_id(&self, slot: PaperDoll) -> i32 {
        self.paperdoll[slot as usize][0]
    }

    #[must_use]
    pub fn get_paper_doll_visual_id(&self, slot: PaperDoll) -> i32 {
        self.paperdoll[slot as usize][3]
    }
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_enchant_effect(&self, slot: PaperDoll) -> i16 {
        self.paperdoll[slot as usize][3] as i16
    }
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_enchant_effect_as_byte(&self, slot: PaperDoll) -> u8 {
        let effect = self.get_enchant_effect(slot);
        if effect > 127 {
            127
        } else {
            effect as u8
        }
    }

    pub fn get_hair_style(&self) -> i32 {
        self.char_model
            .variables
            .get("visualHairId")
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }
    pub fn get_hair_color(&self) -> i32 {
        self.char_model
            .variables
            .get("visualHairColorId")
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }
    pub fn get_face(&self) -> i32 {
        self.char_model
            .variables
            .get("visualFaceId")
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    #[must_use]
    pub fn get_max_hp(&self) -> f64 {
        self.char_model.max_hp
    }
    #[must_use]
    pub fn get_max_mp(&self) -> f64 {
        self.char_model.max_mp
    }
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_delete_timer(&self) -> i32 {
        if let Some(delete_at) = self.char_model.delete_at {
            let now = Utc::now();
            let delta = delete_at.with_timezone(&Utc).signed_duration_since(now);
            delta.num_seconds() as i32
        } else {
            0
        }
    }
    #[must_use]
    pub fn get_vitality_used(&self) -> i32 {
        self.char_model
            .variables
            .get(Self::VITALITY_ITEMS_USED_VARIABLE_NAME)
            .and_then(Value::as_i64)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(0)
    }
    #[must_use]
    pub fn is_hair_accessory_enabled(&self) -> bool {
        self.char_model
            .variables
            .get(Self::HAIR_ACCESSORY_VARIABLE_NAME)
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }
    #[must_use]
    pub fn get_transform_id(&self) -> i32 {
        i32::from(self.char_model.transform_id)
    }
}

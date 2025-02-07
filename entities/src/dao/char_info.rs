use crate::entities::{character, item};
use chrono::Utc;
use sea_orm::DbErr;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum CharVariables {
    VisualHairStyleId,
    VisualHairColorId,
    VisualFaceId,
    HairAccessoryEnabled,
    VitalityItemsUsed
}

impl CharVariables {
    #[must_use]
    pub fn as_key(&self) -> &'static str {
        match self {
            CharVariables::VisualHairStyleId => "visualHairStyleId",
            CharVariables::VisualHairColorId => "visualHairColorId",
            CharVariables::VisualFaceId => "visualFaceId",
            CharVariables::HairAccessoryEnabled => "hairAccessoryEnabled",
            CharVariables::VitalityItemsUsed => "vitalityItemsUsed"
        }
    }
}

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
}

#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub char_model: character::Model,
    pub items: Vec<item::Model>,
    pub paperdoll: [[i32; 4]; 33],
}

#[allow(clippy::missing_errors_doc)]
impl CharacterInfo {
    pub fn new(char_model: character::Model, items: Vec<item::Model>) -> Result<Self, DbErr> {
        let paperdoll = char_model.restore_visible_inventory(&items)?;
        Ok(Self {
            char_model,
            items,
            paperdoll,
        })
    }
    #[must_use]
    pub fn paper_doll_item_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[1])
    }

    pub fn try_get_paper_doll_item_id(&self, slot: PaperDoll) -> anyhow::Result<i32> {
        self.paper_doll_item_id(slot)
            .ok_or(anyhow::anyhow!("No paper doll item id at slot {slot:?}"))
    }

    #[must_use]
    pub fn get_item(&self, item_obj_id: i32) -> Option<&item::Model> {
        self.items.iter().find(|i| i.id == item_obj_id)
    }

    #[must_use]
    pub fn get_weapon(&self) -> Option<&item::Model> {
        if let Some(r_id) = self.get_paper_doll_object_id(PaperDoll::RHand) {
            return self.get_item(r_id);
        }
        None
    }
    pub fn try_get_race(&self) -> anyhow::Result<Race> {
        Race::try_from(self.char_model.race_id)
    }

    #[must_use]
    pub fn get_paper_doll_object_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[0])
    }

    #[must_use]
    pub fn get_paper_doll_visual_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[3])
    }
    pub fn try_get_paper_doll_visual_id(&self, slot: PaperDoll) -> anyhow::Result<i32> {
        self.get_paper_doll_visual_id(slot)
            .ok_or(anyhow::anyhow!("No paperdoll at slot {slot:?}"))
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
            .get(CharVariables::VisualHairStyleId.as_key())
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    pub fn get_hair_color(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VisualHairColorId.as_key())
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    pub fn get_face(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VisualFaceId.as_key())
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
            let time_left = delta.num_seconds() as i32;
            if time_left >= 0 {
                return time_left;
            }
        }
        0
    }
    #[must_use]
    pub fn get_vitality_used(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VitalityItemsUsed.as_key())
            .and_then(Value::as_i64)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(0)
    }
    #[must_use]
    pub fn is_hair_accessory_enabled(&self) -> bool {
        self.char_model
            .variables
            .get(CharVariables::HairAccessoryEnabled.as_key())
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }
    #[must_use]
    pub fn get_transform_id(&self) -> i32 {
        i32::from(self.char_model.transform_id)
    }
}

#[repr(i32)]
#[derive(Clone, Debug, Copy, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub enum Race {
    Human,
    Elf,
    DarkElf,
    Orc,
    Dwarf,
    Kamael,
    Ertheia,
    Animal,
    Beast,
    Bug,
    CastleGuard,
    Construct,
    Demonic,
    Divine,
    Dragon,
    Elemental,
    Etc,
    Fairy,
    Giant,
    Humanoid,
    Mercenary,
    None,
    Plant,
    SiegeWeapon,
    Undead,
    Friend,
}

fn try_from(value: i32) -> anyhow::Result<Race> {
    use anyhow::bail;
    #[allow(clippy::enum_glob_use)]
    use Race::*;
    match value {
        0 => Ok(Human),
        1 => Ok(Elf),
        2 => Ok(DarkElf),
        3 => Ok(Orc),
        4 => Ok(Dwarf),
        5 => Ok(Kamael),
        6 => Ok(Ertheia),
        7 => Ok(Animal),
        8 => Ok(Beast),
        9 => Ok(Bug),
        10 => Ok(CastleGuard),
        11 => Ok(Construct),
        12 => Ok(Demonic),
        13 => Ok(Divine),
        14 => Ok(Dragon),
        15 => Ok(Elemental),
        16 => Ok(Etc),
        17 => Ok(Fairy),
        18 => Ok(Giant),
        19 => Ok(Humanoid),
        20 => Ok(Mercenary),
        21 => Ok(None),
        22 => Ok(Plant),
        23 => Ok(SiegeWeapon),
        24 => Ok(Undead),
        _ => bail!("Unknown race value: {}", value),
    }
}

impl TryFrom<i32> for Race {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> anyhow::Result<Self> {
        try_from(value)
    }
}

impl TryFrom<i8> for Race {
    type Error = anyhow::Error;
    fn try_from(value: i8) -> anyhow::Result<Self> {
        try_from(value.into())
    }
}

impl TryFrom<u8> for Race {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> anyhow::Result<Self> {
        try_from(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::item::LocType;
    use crate::entities::user;
    use sea_orm::{ActiveModelTrait, ActiveValue, JsonValue, TryIntoModel};
    use serde_json::json;
    use test_utils::utils::get_test_db;

    #[test]
    fn test_get_paperdoll_ordered_ids() {
        let ids = PaperDoll::ordered_ids();
        let expected_ids = [
            PaperDoll::Under,
            PaperDoll::Rear,
            PaperDoll::Lear,
            PaperDoll::Neck,
            PaperDoll::RFinger,
            PaperDoll::LFinger,
            PaperDoll::Head,
            PaperDoll::RHand,
            PaperDoll::LHand,
            PaperDoll::Gloves,
            PaperDoll::Chest,
            PaperDoll::Legs,
            PaperDoll::Feet,
            PaperDoll::Cloak,
            PaperDoll::RHand,
            PaperDoll::Hair,
            PaperDoll::Hair2,
            PaperDoll::RBracelet,
            PaperDoll::LBracelet,
            PaperDoll::Deco1,
            PaperDoll::Deco2,
            PaperDoll::Deco3,
            PaperDoll::Deco4,
            PaperDoll::Deco5,
            PaperDoll::Deco6,
            PaperDoll::Belt,
            PaperDoll::Brooch,
            PaperDoll::BroochJewel1,
            PaperDoll::BroochJewel2,
            PaperDoll::BroochJewel3,
            PaperDoll::BroochJewel4,
            PaperDoll::BroochJewel5,
            PaperDoll::BroochJewel6,
        ];
        assert_eq!(ids, expected_ids);
    }
    #[test]
    fn test_get_paperdoll_visual_ids() {
        let ids = PaperDoll::visual_ids();
        let expected_ids = [
            PaperDoll::RHand,
            PaperDoll::LHand,
            PaperDoll::Gloves,
            PaperDoll::Chest,
            PaperDoll::Legs,
            PaperDoll::Feet,
            PaperDoll::RHand,
            PaperDoll::Hair,
            PaperDoll::Hair2,
        ];
        assert_eq!(ids, expected_ids);
    }
    #[test]
    fn test_valid_race_values() {
        use Race::*;
        let test_cases = vec![
            (0, Human),
            (1, Elf),
            (2, DarkElf),
            (3, Orc),
            (4, Dwarf),
            (5, Kamael),
            (6, Ertheia),
            (7, Animal),
            (8, Beast),
            (9, Bug),
            (10, CastleGuard),
            (11, Construct),
            (12, Demonic),
            (13, Divine),
            (14, Dragon),
            (15, Elemental),
            (16, Etc),
            (17, Fairy),
            (18, Giant),
            (19, Humanoid),
            (20, Mercenary),
            (21, None),
            (22, Plant),
            (23, SiegeWeapon),
            (24, Undead),
        ];

        for (input, expected) in test_cases {
            assert_eq!(try_from(input).unwrap(), expected);
        }
    }
    #[test]
    fn test_invalid_race_values() {
        let invalid_values = vec![-1, 25, 100, i32::MAX, i32::MIN];

        for &value in &invalid_values {
            assert!(
                try_from(value).is_err(),
                "Expected error for value: {value}"
            );
        }
    }

    #[tokio::test]
    async fn test_get_char_info() {
        let db_pool = get_test_db().await;
        let user = user::ActiveModel {
            id: ActiveValue::NotSet,
            username: ActiveValue::Set("admin".to_string()),
            access_level: ActiveValue::Set(0),
            ban_ip: ActiveValue::NotSet,
            password: ActiveValue::Set("hashed_pass".to_string()),
            ban_duration: ActiveValue::NotSet,
        }
        .save(&db_pool)
        .await
        .unwrap()
        .try_into_model()
        .unwrap();
        let now = Utc::now();
        let char = character::ActiveModel {
            name: ActiveValue::Set("Admin".to_string()),
            level: ActiveValue::Set(1),
            face: ActiveValue::Set(2),
            hair_style: ActiveValue::Set(2),
            x: ActiveValue::Set(0),
            y: ActiveValue::Set(0),
            z: ActiveValue::Set(0),
            transform_id: ActiveValue::Set(2),
            variables: ActiveValue::Set(json!({
                CharVariables::VitalityItemsUsed.as_key(): 10,
                CharVariables::HairAccessoryEnabled.as_key(): false,
                CharVariables::VisualHairColorId.as_key(): 4,
                CharVariables::VisualHairStyleId.as_key(): 3,
            })),
            delete_at: ActiveValue::Set(Some(now.into())),
            class_id: ActiveValue::Set(1),
            race_id: ActiveValue::Set(Race::Human as i8),
            hair_color: ActiveValue::Set(0),
            is_female: ActiveValue::Set(false),
            user_id: ActiveValue::Set(user.id),
            ..Default::default()
        }
        .save(&db_pool)
        .await
        .unwrap()
        .try_into_model()
        .unwrap();
        let items = vec![
            item::Model {
                id: 1,
                owner: char.id,
                item_id: 2,
                count: 1,
                enchant_level: 3,
                loc: LocType::Paperdoll,
                variables: JsonValue::default(),
                variations: JsonValue::default(),
                loc_data: PaperDoll::RHand as i32,
                ..Default::default()
            },
            item::Model {
                id: 2,
                owner: char.id,
                item_id: 2,
                count: 1,
                enchant_level: 0,
                loc: LocType::Paperdoll,
                variables: JsonValue::default(),
                variations: JsonValue::default(),
                loc_data: PaperDoll::Hair as i32,
                ..Default::default()
            },
        ];
        let char_info = CharacterInfo::new(char, items).unwrap();
        let weapon = char_info.get_weapon().unwrap();
        //todo more asserts
        assert_eq!(weapon.id, 1);
        assert!(!char_info.is_hair_accessory_enabled());
        assert_eq!(char_info.get_hair_color(),4);
        assert_eq!(char_info.get_hair_style(),3);
        assert_eq!(char_info.get_transform_id(), 2);
        assert_eq!(char_info.get_delete_timer(), 0);
        assert_eq!(char_info.get_vitality_used(), 10);
    }
}

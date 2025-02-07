use crate::entities::item;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde_json::Value;

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Copy, PartialEq, Eq, Default)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "LocType")]
pub enum LocType {
    #[default]
    #[sea_orm(string_value = "INVENTORY")]
    Inventory,
    #[sea_orm(string_value = "PAPERDOLL")]
    Paperdoll,
    #[sea_orm(string_value = "NPC")]
    Npc,
    #[sea_orm(string_value = "CLANWH")]
    ClanWh,
    #[sea_orm(string_value = "PET")]
    Pet,
    #[sea_orm(string_value = "WAREHOUSE")]
    Warehouse,
    #[sea_orm(string_value = "VOID")]
    Void,
    #[sea_orm(string_value = "PET_EQUIP")]
    PetEquip,
    #[sea_orm(string_value = "LEASE")]
    Lease,
    #[sea_orm(string_value = "REFUND")]
    Refund,
    #[sea_orm(string_value = "MAIL")]
    Mail,
    #[sea_orm(string_value = "FREIGHT")]
    Freight,
    #[sea_orm(string_value = "COMMISSION")]
    Commission,
}

#[allow(clippy::missing_errors_doc)]
impl item::Model {
    pub const VISUAL_ID: &'static str = "visualId";
    pub const VISUAL_APPEARANCE_STONE_ID: &'static str = "visualAppearanceStoneId";
    pub const VISUAL_APPEARANCE_LIFE_TIME: &'static str = "visualAppearanceLifetime";
    ///
    /// # returns
    ///  tuple where:
    ///  1. mineral id
    ///  2. option 1
    ///  3. option 2
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_augmentation(&self) -> Option<(i32, i32, i32)> {
        let option_1 = self.variations.get("option_1").and_then(Value::as_i64)?;
        let option_2 = self.variations.get("option_2").and_then(Value::as_i64)?;
        let mineral_id = self
            .variations
            .get("mineralId")
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32;
        if option_1 > -1 && option_2 > -1 {
            return Some((mineral_id, option_1 as i32, option_2 as i32));
        }
        None
    }
}

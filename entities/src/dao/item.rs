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

#[derive(Debug, Clone)]
pub enum ItemVariations {
    MineralId,
    Option1,
    Option2,
}
impl ItemVariations {
    #[must_use]
    pub fn as_key(&self) -> &'static str {
        match self {
            ItemVariations::MineralId => "mineral_id",
            ItemVariations::Option1 => "option1",
            ItemVariations::Option2 => "option2",
        }
    }
}
#[derive(Debug, Clone)]
pub enum ItemVariables{
    VisualId,
    VisualAppearanceStoneId,
    VisualAppearanceLifeTime
}
impl ItemVariables {
    #[must_use]
    pub fn as_key(&self) -> &'static str {
        match self {
            ItemVariables::VisualId => "visual_id",
            ItemVariables::VisualAppearanceStoneId => "visual_appearance_stone_id",
            ItemVariables::VisualAppearanceLifeTime => "visual_appearance_lifetime",
        }
    }
}
#[allow(clippy::missing_errors_doc)]
impl item::Model {
    ///
    /// # returns
    ///  tuple where:
    ///  1. mineral id
    ///  2. option 1
    ///  3. option 2
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_augmentation(&self) -> Option<(i32, i32, i32)> {
        let option_1 = self
            .variations
            .get(ItemVariations::Option1.as_key())
            .and_then(Value::as_i64)?;
        let option_2 = self
            .variations
            .get(ItemVariations::Option2.as_key())
            .and_then(Value::as_i64)?;
        let mineral_id = self
            .variations
            .get(ItemVariations::MineralId.as_key())
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32;
        if option_1 > -1 && option_2 > -1 {
            return Some((mineral_id, option_1 as i32, option_2 as i32));
        }
        None
    }
}

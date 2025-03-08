use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Copy, PartialEq, Eq, Default)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "CastleSide")]
pub enum CastleSide {
    #[default]
    #[sea_orm(string_value = "NEUTRAL")]
    Neutral,
    #[sea_orm(string_value = "LIGHT")]
    Light,
    #[sea_orm(string_value = "DARK")]
    Dark,
    
}
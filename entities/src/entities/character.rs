//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Default, DeriveEntityModel)]
#[sea_orm(table_name = "character")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub level: i8,
    pub delete_at: Option<DateTimeWithTimeZone>,
    pub user_id: i32,
    #[sea_orm(column_type = "Double")]
    pub max_hp: f64,
    #[sea_orm(column_type = "Double")]
    pub cur_hp: f64,
    #[sea_orm(column_type = "Double")]
    pub max_cp: f64,
    #[sea_orm(column_type = "Double")]
    pub cur_cp: f64,
    #[sea_orm(column_type = "Double")]
    pub cur_mp: f64,
    #[sea_orm(column_type = "Double")]
    pub max_mp: f64,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub is_female: bool,
    pub heading: Option<i32>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub exp: i64,
    pub exp_before_death: Option<i64>,
    pub sp: i64,
    pub reputation: i32,
    pub fame: i32,
    pub rb_points: i32,
    pub pvp_kills: i32,
    pub pk_kills: i32,
    pub race_id: i8,
    pub class_id: i8,
    pub base_class_id: i8,
    pub transform_id: i16,
    pub can_craft: Option<bool>,
    pub title: Option<String>,
    pub title_color: Option<i32>,
    pub access_level: i32,
    pub online: Option<i8>,
    pub online_time: Option<i32>,
    pub char_slot: Option<i8>,
    pub last_access: Option<DateTimeWithTimeZone>,
    pub clan_privs: Option<i32>,
    pub wants_peace: Option<i8>,
    pub power_grade: Option<i8>,
    pub nobless: bool,
    pub sub_pledge: Option<i16>,
    pub lvl_joined_academy: i8,
    pub apprentice: i32,
    pub sponsor: i32,
    pub clan_join_expiry_time: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_type = "JsonBinary")]
    pub variables: Json,
    pub clan_create_expiry_time: Option<DateTimeWithTimeZone>,
    pub bookmark_slot: i16,
    pub vitality_points: i32,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub language: Option<String>,
    pub faction: i8,
    pub pc_cafe_points: i32,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Char {} ( Level {})", self.name, self.level)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::item::Entity")]
    Item,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Item.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

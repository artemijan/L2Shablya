use sea_orm::JsonValue;
use crate::m20220101_000001_create_user as previous;
use sea_orm_migration::schema::{big_unsigned, big_unsigned_null, boolean, double, integer_null, json_binary, small_unsigned, small_unsigned_null, string_len_null, timestamp_with_time_zone_null, tiny_unsigned, tiny_unsigned_null, unsigned, unsigned_null};
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_auto, string},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[allow(clippy::too_many_lines)]
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Character::Table)
                    .if_not_exists()
                    .col(pk_auto(Character::Id))
                    .col(string(Character::Name))
                    .col(tiny_unsigned(Character::Level))
                    .col(timestamp_with_time_zone_null(Character::DeleteAt))
                    .col(integer(Character::UserId))
                    .col(double(Character::MaxHP).default(0))
                    .col(double(Character::CurHP).default(0))
                    .col(double(Character::MaxCP).default(0))
                    .col(double(Character::CurCP).default(0))
                    .col(double(Character::CurMP).default(0))
                    .col(double(Character::MaxMP).default(0))
                    .col(small_unsigned_null(Character::Face))
                    .col(small_unsigned_null(Character::HairStyle))
                    .col(small_unsigned_null(Character::HairColor))
                    .col(small_unsigned(Character::Sex))
                    .col(integer_null(Character::Heading))
                    .col(integer(Character::X))
                    .col(integer(Character::Y))
                    .col(integer(Character::Z))
                    .col(big_unsigned(Character::Exp).default(0))
                    .col(big_unsigned_null(Character::ExpBeforeDeath).default(0))
                    .col(big_unsigned(Character::SP).default(0))
                    .col(integer(Character::Reputation).default(0))
                    .col(unsigned(Character::Fame).default(0))
                    .col(unsigned(Character::RbPoints).default(0))
                    .col(unsigned(Character::PvpKills).default(0))
                    .col(unsigned(Character::PkKills).default(0))
                    .col(tiny_unsigned(Character::RaceId))
                    .col(tiny_unsigned(Character::ClassId).default(0))
                    .col(tiny_unsigned(Character::BaseClassId).default(0))
                    .col(small_unsigned(Character::TransformId))
                    .col(tiny_unsigned_null(Character::CanCraft))
                    .col(string_len_null(Character::Title, 21))
                    .col(unsigned_null(Character::TitleColor).default(15_530_402))
                    .col(unsigned(Character::AccessLevel).default(0))
                    .col(tiny_unsigned_null(Character::Online))
                    .col(unsigned_null(Character::OnlineTime))
                    .col(tiny_unsigned_null(Character::CharSlot))
                    .col(timestamp_with_time_zone_null(Character::LastAccess))
                    .col(unsigned_null(Character::ClanPrivs).default(0))
                    .col(tiny_unsigned_null(Character::WantsPeace).default(0))
                    .col(tiny_unsigned_null(Character::PowerGrade))
                    .col(boolean(Character::Nobless).default(0))
                    .col(small_unsigned_null(Character::SubPledge).default(0))
                    .col(tiny_unsigned(Character::LvlJoinedAcademy).default(0))
                    .col(unsigned(Character::Apprentice).default(0))
                    .col(unsigned(Character::Sponsor).default(0))
                    .col(timestamp_with_time_zone_null(Character::ClanJoinExpiryTime))
                    .col(json_binary(Character::Variables).default(JsonValue::default()))
                    .col(timestamp_with_time_zone_null(
                        Character::ClanCreateExpiryTime,
                    ))
                    .col(small_unsigned(Character::BookmarkSlot).default(0))
                    .col(unsigned(Character::VitalityPoints).default(0))
                    .col(timestamp_with_time_zone_null(Character::CreatedAt))
                    .col(string_len_null(Character::Language, 2))
                    .col(tiny_unsigned(Character::Faction).default(0))
                    .col(integer(Character::PcCafePoints).default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_char_user_id")
                            .from(Character::Table, Character::UserId)
                            .to(previous::User::Table, previous::User::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_name") // Name of the index
                    .table(Character::Table)
                    .col(Character::Name)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Character::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Character {
    Table,
    Id,
    Name,
    UserId,
    Level,
    MaxHP,
    CurHP,
    MaxCP,
    CurCP,
    MaxMP,
    CurMP,
    Face,
    HairStyle,
    HairColor,
    Sex,
    Heading,
    X,
    Y,
    Z,
    Exp,
    ExpBeforeDeath,
    SP,
    Reputation,
    Fame,
    RbPoints,
    PvpKills,
    PkKills,
    //ClanId,
    RaceId,
    ClassId,
    BaseClassId,
    TransformId,
    DeleteAt,
    CanCraft,
    Title,
    TitleColor,
    AccessLevel,
    Online,
    OnlineTime,
    CharSlot,
    LastAccess,
    ClanPrivs,
    WantsPeace,
    PowerGrade,
    Nobless,
    SubPledge,
    LvlJoinedAcademy,
    Apprentice,
    Sponsor,
    ClanJoinExpiryTime,
    ClanCreateExpiryTime,
    BookmarkSlot,
    VitalityPoints,
    CreatedAt,
    Language,
    Faction,
    PcCafePoints,
    Variables
}

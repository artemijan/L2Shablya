use sea_orm_migration::{prelude::*, schema::*};
use sea_orm::JsonValue;
use crate::m20241213_210106_create_char::Character;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Item::Table)
                    .if_not_exists()
                    .col(pk_auto(Item::Id))
                    .col(integer(Item::Owner))
                    .col(integer(Item::ItemId))
                    .col(big_unsigned(Item::Count).default(0))
                    .col(integer(Item::EnchantLevel).default(0))
                    .col(string(Item::Loc))
                    .col(json_binary(Item::Variables).default(JsonValue::default()))
                    .col(json_binary(Item::Variations).default(JsonValue::default()))
                    .col(integer(Item::LocData))
                    .col(integer(Item::TimeOfUse))
                    .col(integer(Item::CustomType1).default(0))
                    .col(integer(Item::CustomType2).default(0))
                    .col(decimal_len(Item::ManaLeft, 5, 0).default(-1))
                    .col(decimal_len(Item::Time, 13, 0).default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_owner_char_id")
                            .from(Item::Table, Item::Owner)
                            .to(Character::Table, Character::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_loc") // Name of the index
                    .table(Item::Table)
                    .col(Item::Loc)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_owner_loc") // Name of the index
                    .table(Item::Table)
                    .col(Item::Owner)
                    .col(Item::Loc)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Item::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Item {
    Table,
    Id,
    Owner,
    ItemId,
    Count,
    EnchantLevel,
    Loc,
    LocData,
    TimeOfUse,
    CustomType1,
    CustomType2,
    ManaLeft,
    Time,
    Variables,
    Variations
}

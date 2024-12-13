use crate::m20220101_000001_create_user as previous;
use sea_orm_migration::{
    prelude::*,
    schema::{date_time_null, integer, pk_auto, string},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

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
                    .col(integer(Character::Level))
                    .col(date_time_null(Character::DeleteAt))
                    .col(ColumnDef::new(Character::UserId).integer().not_null())
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
enum Character {
    Table,
    Id,
    Name,
    UserId,
    Level,
    DeleteAt,
}

use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_auto, string, string_null},
};
use sea_orm_migration::schema::big_integer_null;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Username))
                    .col(integer(User::AccessLevel))
                    .col(string_null(User::BanIp))
                    .col(string(User::Password))
                    .col(big_integer_null(User::BanDuration))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_username") // Name of the index
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    AccessLevel,
    BanDuration,
    BanIp,
    Password
}

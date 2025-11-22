use crate::m20250302_182532_create_clan::ClanAlly;
use sea_orm::DbBackend;
use sea_orm_migration::{prelude::*, schema::{pk_auto, binary_len, tiny_integer, integer_null}};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Crest::Table)
                    .if_not_exists()
                    .col(pk_auto(Crest::Id))
                    .col(binary_len(Crest::Data, 2176))
                    .col(tiny_integer(Crest::Type))
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        match manager.get_database_backend() {
            DbBackend::Postgres => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(ClanAlly::Table)
                            .add_column_if_not_exists(integer_null(ClanAlly::CrestId))
                            .add_column_if_not_exists(integer_null(ClanAlly::CrestLargeId))
                            .add_column_if_not_exists(integer_null(ClanAlly::AllyCrestId))
                            .add_foreign_key(
                                TableForeignKey::new()
                                    .name("clan_ally_crest_id_fk")
                                    .from_tbl(ClanAlly::Table)
                                    .from_col(ClanAlly::CrestId)
                                    .to_tbl(Crest::Table)
                                    .to_col(Crest::Id),
                            )
                            .add_foreign_key(
                                TableForeignKey::new()
                                    .name("clan_ally_crest_large_id_fk")
                                    .from_tbl(ClanAlly::Table)
                                    .from_col(ClanAlly::CrestLargeId)
                                    .to_tbl(Crest::Table)
                                    .to_col(Crest::Id),
                            )
                            .add_foreign_key(
                                TableForeignKey::new()
                                    .name("clan_ally_ally_crest_id_fk")
                                    .from_tbl(ClanAlly::Table)
                                    .from_col(ClanAlly::AllyCrestId)
                                    .to_tbl(Crest::Table)
                                    .to_col(Crest::Id),
                            )
                            .to_owned(),
                    )
                    .await
            }
            DbBackend::Sqlite => {
                if !manager.has_column("clan_ally", "crest_id").await? {
                    db.execute_unprepared(
                        "ALTER TABLE `clan_ally` ADD COLUMN crest_id integer NULL REFERENCES crest (id) ON DELETE SET NULL;"
                    ).await?;
                }
                if !manager.has_column("clan_ally", "crest_large_id").await? {
                    db.execute_unprepared(
                        "ALTER TABLE `clan_ally` ADD COLUMN crest_large_id integer NULL REFERENCES crest (id) ON DELETE SET NULL;"
                    ).await?;
                }
                if !manager.has_column("clan_ally", "ally_crest_id").await? {
                    db.execute_unprepared(
                        "ALTER TABLE `clan_ally` ADD COLUMN ally_crest_id integer NULL REFERENCES crest (id) ON DELETE SET NULL;"
                    ).await?;
                }
                Ok(())
            }
            _ => Err(DbErr::Migration("Unsupported database backend".to_string())),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClanAlly::Table)
                    .drop_column(ClanAlly::CrestId)
                    .drop_column(ClanAlly::CrestLargeId)
                    .drop_column(ClanAlly::AllyCrestId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Crest::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Crest {
    Table,
    Id,
    Data,
    Type,
}

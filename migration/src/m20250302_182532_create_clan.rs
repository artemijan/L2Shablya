use crate::m20241213_210106_create_char::Character;
use sea_orm::DbBackend;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
const LEADER_ID_FOREIGN_KEY_NAME: &str = "fk_leader_clan_id";
const ALLY_ID_FOREIGN_KEY_NAME: &str = "fk_ally_id_clan";
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClanAlly::Table)
                    .if_not_exists()
                    .col(pk_auto(ClanAlly::Id))
                    .col(string_len(ClanAlly::Name, 45))
                    .col(boolean(ClanAlly::IsAlly).default(false))
                    .col(integer_null(ClanAlly::AllyId))
                    .col(tiny_unsigned(ClanAlly::Level).default(0))
                    .col(unsigned(ClanAlly::Reputation).default(0))
                    .col(timestamp_with_time_zone_null(ClanAlly::CreatedAt))
                    .col(small_unsigned(ClanAlly::BloodAllianceCount).default(0))
                    .col(small_unsigned(ClanAlly::BloodOathCount).default(0))
                    .col(integer(ClanAlly::LeaderId).default(0))
                    .col(timestamp_with_time_zone_null(ClanAlly::AuctionBidAt))
                    .col(timestamp_with_time_zone_null(
                        ClanAlly::AllyPenaltyExpiryTime,
                    ))
                    .col(tiny_unsigned(ClanAlly::AllyPenaltyType).default(0))
                    .col(timestamp_with_time_zone_null(
                        ClanAlly::CharPenaltyExpiryTime,
                    ))
                    .col(timestamp_with_time_zone_null(
                        ClanAlly::DissolvingExpiryTime,
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name(ALLY_ID_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::SetNull)
                            .from(ClanAlly::Table, ClanAlly::AllyId)
                            .to(ClanAlly::Table, ClanAlly::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(LEADER_ID_FOREIGN_KEY_NAME)
                            .from_tbl(ClanAlly::Table)
                            .from_col(ClanAlly::LeaderId)
                            .to_tbl(Character::Table)
                            .to_col(Character::Id),
                    )
                    .to_owned(),
            )
            .await?;
        match manager.get_database_backend() {
            DbBackend::Postgres => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Character::Table)
                            .add_column_if_not_exists(integer_null(Character::ClanId))
                            .add_foreign_key(
                                TableForeignKey::new()
                                    .name("character_clan_id_fk")
                                    .from_tbl(Character::Table)
                                    .from_col(Character::ClanId)
                                    .to_tbl(ClanAlly::Table)
                                    .to_col(ClanAlly::Id),
                            )
                            .to_owned(),
                    )
                    .await
            }
            DbBackend::Sqlite => {
                if !manager.has_column("character", "clan_id").await? {
                    let db = manager.get_connection();
                    db.execute_unprepared(
                        "ALTER TABLE `character` ADD COLUMN clan_id integer NULL REFERENCES clan_ally (id) ON DELETE SET NULL;"
                    ).await?;
                }
                Ok(())
            }
            _ => Err(DbErr::Migration("Unsupported database backend".to_string())),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Character::Table)
                    .drop_column(Character::ClanId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ClanAlly::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ClanAlly {
    Table,
    Id,
    Name,
    Level,
    Reputation,
    CreatedAt,
    BloodAllianceCount,
    BloodOathCount,
    AllyId,
    IsAlly,
    LeaderId,
    CrestId,
    CrestLargeId,
    AllyCrestId,
    AuctionBidAt,
    AllyPenaltyExpiryTime,
    AllyPenaltyType,
    CharPenaltyExpiryTime,
    DissolvingExpiryTime,
}

use crate::m20250302_182532_create_clan::ClanAlly;
use sea_orm_migration::{prelude::*, schema::{pk_auto, string, integer, big_unsigned, timestamp_with_time_zone_null, boolean, small_unsigned}};

#[derive(DeriveMigrationName)]
pub struct Migration;
const CLAN_ID_FOREIGN_KEY_NAME: &str = "fk_castle_clan_id";
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Castle::Table)
                    .if_not_exists()
                    .col(pk_auto(Castle::Id))
                    .col(string(Castle::Name))
                    .col(integer(Castle::ClanId))
                    .col(string(Castle::Side).default("NEUTRAL"))
                    .col(big_unsigned(Castle::Treasury).default(0))
                    .col(timestamp_with_time_zone_null(Castle::SiegeStartsAt))
                    .col(timestamp_with_time_zone_null(Castle::RegistrationEndsAt))
                    .col(boolean(Castle::ShowNpcCrest).default(true))
                    .col(small_unsigned(Castle::TicketBuyCount).default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name(CLAN_ID_FOREIGN_KEY_NAME)
                            .from(Castle::Table, Castle::ClanId)
                            .to(ClanAlly::Table, ClanAlly::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Castle::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Castle {
    Table,
    Id,
    Name,
    Side,
    ClanId,
    Treasury,
    SiegeStartsAt,
    RegistrationEndsAt,
    ShowNpcCrest,
    TicketBuyCount,
}

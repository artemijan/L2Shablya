use crate::m20241213_210106_create_char::Character;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const CHAR_ID_FOREIGN_KEY_NAME: &str = "char_id_quest_foreign_key";
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Quest::Table)
                    .if_not_exists()
                    .col(integer(Quest::CharId))
                    .col(string(Quest::Name))
                    .col(string(Quest::Var))
                    .col(string(Quest::Value))
                    .primary_key(
                        Index::create()
                            .col(Quest::CharId)
                            .col(Quest::Name)
                            .col(Quest::Var),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(CHAR_ID_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::Cascade)
                            .from(Quest::Table, Quest::CharId)
                            .to(Character::Table, Character::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Quest::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Quest {
    Table,
    CharId,
    Name,
    Var,
    Value,
}

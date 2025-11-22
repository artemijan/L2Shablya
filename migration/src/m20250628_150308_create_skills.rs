use crate::m20241213_210106_create_char::Character;
use sea_orm_migration::{prelude::*, schema::{integer, small_integer}};

#[derive(DeriveMigrationName)]
pub struct Migration;

const CHAR_ID_FOREIGN_KEY_NAME: &str = "fk_char_id_skill";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Skill::Table)
                    .if_not_exists()
                    .col(integer(Skill::Id))
                    .col(integer(Skill::CharId))
                    .col(small_integer(Skill::Level))
                    .col(small_integer(Skill::SubLevel))
                    .col(integer(Skill::ClassIndex))
                    .foreign_key(
                        ForeignKey::create()
                            .name(CHAR_ID_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::Cascade)
                            .from(Skill::Table, Skill::CharId)
                            .to(Character::Table, Character::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(Skill::CharId)
                            .col(Skill::ClassIndex)
                            .col(Skill::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Skill::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Skill {
    Table,
    Id,
    CharId,
    Level,
    SubLevel,
    ClassIndex,
}

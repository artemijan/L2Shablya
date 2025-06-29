use crate::m20241213_210106_create_char::Character;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const SENDER_MAIL_FOREIGN_KEY_NAME: &str = "fk_sender_mail";
const RECIPIENT_MAIL_FOREIGN_KEY_NAME: &str = "fk_recipient_mail";
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CharacterMail::Table)
                    .if_not_exists()
                    .col(pk_auto(CharacterMail::Id))
                    .col(string(CharacterMail::Subject))
                    .col(string_null(CharacterMail::Message))
                    .col(string(CharacterMail::Items))
                    .col(integer_null(CharacterMail::Sender))
                    .col(integer_null(CharacterMail::Recipient))
                    .foreign_key(
                        ForeignKey::create()
                            .name(SENDER_MAIL_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::SetNull)
                            .from(CharacterMail::Table, CharacterMail::Sender)
                            .to(Character::Table, Character::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(RECIPIENT_MAIL_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::SetNull)
                            .from(CharacterMail::Table, CharacterMail::Recipient)
                            .to(Character::Table, Character::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CharacterMail::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CharacterMail {
    Table,
    Id,
    Subject,
    Message,
    Items,
    Sender,
    Recipient,
}

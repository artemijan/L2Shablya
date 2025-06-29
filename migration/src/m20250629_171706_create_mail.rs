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
                    .table(PlayerMail::Table)
                    .if_not_exists()
                    .col(pk_auto(PlayerMail::Id))
                    .col(string(PlayerMail::Subject))
                    .col(string_null(PlayerMail::Message))
                    .col(string(PlayerMail::Items))
                    .col(integer_null(PlayerMail::Sender))
                    .col(integer_null(PlayerMail::Recipient))
                    .foreign_key(
                        ForeignKey::create()
                            .name(SENDER_MAIL_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::SetNull)
                            .from(PlayerMail::Table, PlayerMail::Sender)
                            .to(Character::Table, Character::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(RECIPIENT_MAIL_FOREIGN_KEY_NAME)
                            .on_delete(ForeignKeyAction::SetNull)
                            .from(PlayerMail::Table, PlayerMail::Recipient)
                            .to(Character::Table, Character::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerMail::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerMail {
    Table,
    Id,
    Subject,
    Message,
    Items,
    Sender,
    Recipient,
}

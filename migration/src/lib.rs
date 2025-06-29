pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user;
mod m20241213_210106_create_char;
mod m20250103_134559_create_items;
mod m20250302_182532_create_clan;
mod m20250302_194521_create_castle;
mod m20250302_195001_create_crest;
mod m20250628_150308_create_skills;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user::Migration),
            Box::new(m20241213_210106_create_char::Migration),
            Box::new(m20250103_134559_create_items::Migration),
            Box::new(m20250302_182532_create_clan::Migration),
            Box::new(m20250302_194521_create_castle::Migration),
            Box::new(m20250302_195001_create_crest::Migration),
            Box::new(m20250628_150308_create_skills::Migration),
        ]
    }
}

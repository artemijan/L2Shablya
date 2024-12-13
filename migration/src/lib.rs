pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user;
mod m20241213_210106_create_char;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user::Migration),
            Box::new(m20241213_210106_create_char::Migration),
        ]
    }
}

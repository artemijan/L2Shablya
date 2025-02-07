use sea_orm::DatabaseConnection;

pub mod entities;
pub mod dao;

#[cfg(feature = "test-factories")]
pub mod test_factories;

pub type DBPool = DatabaseConnection;
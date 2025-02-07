use sea_orm::DatabaseConnection;

pub mod entities;
pub mod dao;

#[cfg(feature = "test-utils")]
pub mod test_factories;


pub type DBPool = DatabaseConnection;
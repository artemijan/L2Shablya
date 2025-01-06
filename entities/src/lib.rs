use sea_orm::DatabaseConnection;

pub mod entities;
pub mod dao;

pub type DBPool = DatabaseConnection;
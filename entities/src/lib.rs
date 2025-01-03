use sea_orm::DatabaseConnection;

pub mod entities;
mod services;

pub type DBPool = DatabaseConnection;
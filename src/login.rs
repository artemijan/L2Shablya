use crate::common::dto::config::ServerConfig;
use crate::database::new_db_pool;
use crate::login_server::controller::LoginController;
use crate::login_server::PacketHandler;
use anyhow::Context;
use common::network;
use login_server::event_loops;
use sqlx::any::install_default_drivers;
use sqlx::Connection;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::sync::Arc;

mod common;
mod crypt;
mod database;
mod login_server;
mod packet;

#[tokio::main]
pub async fn main() {
    install_default_drivers();
    let config = Arc::new(ServerConfig::load("config/login.yaml"));
    let pool = new_db_pool(&config.server.database).await;
    let lc = Arc::new(LoginController::new());
    database::run_migrations(&pool).await;
    let clients_handle = network::create_handle(
        config.clone(),
        lc.clone(),
        pool.clone(),
        event_loops::client::start,
    );
    let gs_handle = network::create_handle(
        config.clone(),
        lc.clone(),
        pool.clone(),
        event_loops::game_server::start,
    );
    clients_handle.await.unwrap();
    gs_handle.await.unwrap(); // actually this line is never reached, because in previous handle it's infinite loop
}

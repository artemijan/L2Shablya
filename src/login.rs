use crate::common::dto::config;
use crate::database::new_db_pool;
use crate::login_server::controller::Login;
use crate::login_server::PacketHandler;
use anyhow::Context;
use common::network;
use dotenvy::dotenv;
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

///
/// # Panics
///
/// - when Game server thread quits, and so login server can not accept new servers
/// - when client listener thread quits so we can't accept new clients
#[tokio::main]
pub async fn main() {
    install_default_drivers();
    dotenv().ok();
    let config = Arc::new(config::Server::load("config/login.yaml"));
    let pool = new_db_pool(&config.database).await;
    let lc = Arc::new(Login::new(config.clone()));
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

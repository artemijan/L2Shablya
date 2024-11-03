use crate::common::dto::config;
use crate::database::new_db_pool;
use crate::login_server::controller::Login;
use crate::login_server::traits::PacketHandler;
use anyhow::Context;
use common::network;
use dotenvy::dotenv;
use login_server::main_loop;
use sqlx::any::install_default_drivers;
use sqlx::Connection;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::gs_thread::GSHandler;

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
        main_loop::<ClientHandler>,
    );

    let gs_handle = network::create_handle(
        config.clone(),
        lc.clone(),
        pool.clone(),
        main_loop::<GSHandler>,
    );
    clients_handle.await.unwrap();
    gs_handle.await.unwrap(); // actually this line is never reached, because in previous handle it's infinite loop
}

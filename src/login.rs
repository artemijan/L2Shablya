use crate::common::traits::server::Server;
use crate::common::traits::ServerConfig;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::controller::Login;
use crate::login_server::gs_thread::GSHandler;
use crate::login_server::LoginServer;
use anyhow::Context;
use common::traits::handlers::PacketHandler;
use sqlx::Connection;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::sync::Arc;

mod common;
mod crypt;
mod database;
mod login_server;

///
/// # Panics
/// - when can't open a socket
/// - when config file not found
/// - when DB is not accessible
/// - when can't run migrations
///
pub fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .init();
    LoginServer::bootstrap("config/login.yaml", |cfg, db_pool| async move {
        let lc = Arc::new(Login::new(cfg.clone()));
        let clients_handle =
            LoginServer::listener_loop::<ClientHandler>(cfg.clone(), lc.clone(), db_pool.clone());

        let gs_handle =
            LoginServer::listener_loop::<GSHandler>(cfg.clone(), lc.clone(), db_pool.clone());

        clients_handle
            .await
            .unwrap_or_else(|_| panic!("Client handler exited unexpectedly"));
        // actually this line is never reached, because in previous handle it's infinite loop
        gs_handle
            .await
            .unwrap_or_else(|_| panic!("Game server handler exited unexpectedly"));
    });
}

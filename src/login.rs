use crate::common::traits::server::Server;
use crate::common::traits::ServerConfig;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::controller::Login;
use crate::login_server::gs_thread::GSHandler;
use crate::login_server::LoginServer;
use anyhow::Context;
use common::network;
use common::traits::handler::PacketHandler;
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
///
/// - when Game server thread quits, and so login server can not accept new servers
/// - when client listener thread quits so we can't accept new clients
pub fn main() {
    LoginServer::bootstrap("config/login.yaml", |cfg, db_pool| async move {
        let lc = Arc::new(Login::new(cfg.clone()));
        let clients_handle =
            LoginServer::handler_loop::<ClientHandler>(cfg.clone(), lc.clone(), db_pool.clone());

        let gs_handle =
            LoginServer::handler_loop::<GSHandler>(cfg.clone(), lc.clone(), db_pool.clone());
        
        clients_handle.await;
        // actually this line is never reached, because in previous handle it's infinite loop
        gs_handle.await;
    });
}

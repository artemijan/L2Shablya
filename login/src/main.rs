use l2_core::config::login;
use l2_core::traits::server::Server;
use crate::client_thread::ClientHandler;
use crate::controller::Login;
use crate::gs_thread::GSHandler;
use std::sync::Arc;
mod client_thread;
mod controller;
mod gs_thread;
mod packet;
mod dto;
mod message;
pub struct LoginServer;

impl Server for LoginServer {
    type ConfigType = login::LoginServer;
    type ControllerType = Login;
}
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

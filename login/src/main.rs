use crate::client_thread::ClientHandler;
use crate::controller::Login;
use crate::gs_thread::GSHandler;
use l2_core::config::login;
use l2_core::traits::server::Server;
use std::sync::Arc;
use tracing::error;

mod client_thread;
mod controller;
mod dto;
mod gs_thread;
mod message;
mod packet;
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
        let mut clients_handle =
            LoginServer::listener_loop::<ClientHandler>(cfg.clone(), lc.clone(), db_pool.clone());

        let mut gs_handle =
            LoginServer::listener_loop::<GSHandler>(cfg.clone(), lc.clone(), db_pool.clone());

        tokio::select! {
            _ = &mut clients_handle => {
                error!("Client handler exited unexpectedly");
            }
            _ = &mut gs_handle => {
                error!("Game server handler exited unexpectedly");
            }
        }
        if !clients_handle.is_finished() {
            clients_handle.abort();
        }
        if !gs_handle.is_finished() {
            gs_handle.abort();
        }
    });
}

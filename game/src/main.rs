use crate::client_thread::ClientHandler;
use crate::controller::Controller;
use l2_core::config::gs::GSServer;
use l2_core::traits::server::Server;
use std::sync::Arc;
use tracing::error;
use crate::ls_thread::LoginHandler;

mod client_thread;
mod controller;
mod cp_factory;
mod lsp_factory;
mod packets;
mod ls_thread;
pub mod data;
mod tests;
pub mod managers;

pub struct GameServer;

impl Server for GameServer {
    type ConfigType = GSServer;
    type ControllerType = Controller;
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
    GameServer::bootstrap("config/game.yaml", |cfg, db_pool| async move {
        let controller = Arc::new(Controller::new(cfg.clone(), &db_pool).await);
        let mut ls_handle = GameServer::connector_loop::<LoginHandler>(
            cfg.clone(),
            controller.clone(),
            db_pool.clone(),
        );
        let mut client_handle =
            GameServer::listener_loop::<ClientHandler>(cfg, controller, db_pool);
        tokio::select!(
            _ = &mut ls_handle => {
                error!("Login server thread exited unexpectedly");
            },
            _ = &mut client_handle => {
                error!("Client thread exited unexpectedly");
            },
        );
        if !ls_handle.is_finished() {
            ls_handle.abort();
        }
        if !client_handle.is_finished() {
            client_handle.abort();
        }
    });
}

use crate::controller::Controller;
use crate::handlers::LoginHandler;
use l2_core::config::gs::GSServer;
use l2_core::traits::server::Server;
use std::sync::Arc;

mod controller;
mod handlers;
mod lsp_factory;
mod packets;

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
        let controller = Arc::new(Controller::new(cfg.clone()));
        let ls_handle =
            GameServer::connector_loop::<LoginHandler>(cfg.clone(), controller, db_pool.clone());
        ls_handle.await.expect("Login server loop crashed");
    });
}

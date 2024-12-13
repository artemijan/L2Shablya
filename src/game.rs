use crate::common::traits::server::Server;
use crate::game_server::controller::Controller;
use crate::game_server::handlers::LoginHandler;
use crate::game_server::GameServer;
use std::sync::Arc;

mod common;
mod crypt;
mod database;
mod game_server;

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

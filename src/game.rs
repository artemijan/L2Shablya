use crate::common::traits::server::Server;
use crate::game_server::controller::Controller;
use crate::game_server::handlers::PlayerHandler;
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
    GameServer::bootstrap("config/game.yaml", |cfg, db_pool| async move {
        let controller = Arc::new(Controller::new(cfg.clone()));
        let clients_handle =
            GameServer::handler_loop::<PlayerHandler>(cfg.clone(), controller, db_pool.clone());
        clients_handle.await.unwrap();
    });
}

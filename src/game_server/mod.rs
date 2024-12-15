use crate::common::traits::server::Server;
use crate::game_server::controller::Controller;
use crate::common::config::gs::GSServer;
pub mod lsp_factory;
pub mod controller;
pub mod handlers;
pub mod packets;
pub struct GameServer;

impl Server for GameServer {
    type ConfigType = GSServer;
    type ControllerType = Controller;
}

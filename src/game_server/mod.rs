use crate::common::traits::server::Server;
use crate::game_server::controller::Controller;
use crate::game_server::dto::config::GSServer;

pub mod dto;
pub mod controller;
pub mod handlers;

pub struct GameServer;

impl Server for GameServer {
    type ConfigType = GSServer;
    type ControllerType = Controller;
}

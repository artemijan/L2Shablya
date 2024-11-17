use crate::common::traits::handler::PacketHandler;
use crate::common::traits::server::Server;
use crate::common::traits::IpBan;
use crate::login_server::controller::Login;
use crate::login_server::dto::config;
pub mod client_thread;
pub mod controller;
pub mod dto;
pub mod gs_thread;
mod message;
mod packet;

pub struct LoginServer;

impl Server for LoginServer {
    type ConfigType = config::Server;
    type ControllerType = Login;
}

use crate::common::config::login;
use crate::common::traits::handlers::PacketHandler;
use crate::common::traits::server::Server;
use crate::common::traits::IpBan;
use crate::login_server::controller::Login;
pub mod client_thread;
pub mod controller;
pub mod dto;
pub mod gs_thread;
mod message;
mod packet;

pub struct LoginServer;

impl Server for LoginServer {
    type ConfigType = login::LoginServer;
    type ControllerType = Login;
}

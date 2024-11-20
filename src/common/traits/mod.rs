pub mod handlers;
pub mod server;

use crate::common::packet::SendablePacket;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Notify;
use crate::common::dto::{Database, Runtime};

pub trait IpBan {
    fn is_ip_banned(&self, ip: &str) -> bool;
}

pub trait Shutdown {
    fn get_shutdown_listener(&self) -> Arc<Notify>;
    fn shutdown(&self) {
        self.get_shutdown_listener().notify_one();
    }
}

pub trait ServerConfig {
    fn load(path: &str) -> Self;
    fn from_string(content: &str) -> Self;
    fn runtime(&self) -> Option<&Runtime>;
    fn database(&self) -> &Database;
}

use crate::dto::InboundConnection;
use anyhow::Context;
use std::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpSocket};

pub fn bind_addr(config: &InboundConnection) -> anyhow::Result<TcpListener> {
    let addr = format!("{}:{}", &config.ip, &config.port)
        .to_socket_addrs()
        .context(format!(
            "Failed to resolve address {}:{}",
            config.ip, config.port
        ))?
        .next()
        .context("No address found for the given host and port")?;
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(config.reuse_addr)?;
    socket.set_reuseport(config.reuse_port)?;
    socket.set_nodelay(config.no_delay)?;
    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    Ok(listener)
}

use crate::dto::InboundConnection;
use anyhow::Context;
use std::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpSocket};

#[allow(clippy::missing_errors_doc)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::InboundConnection;

    #[tokio::test]
    async fn test_bind_addr() {
        let config = InboundConnection {
            ip: "127.0.0.1".to_string(),
            port: 2106,
            reuse_addr: true,
            reuse_port: true,
            no_delay: true,
        };
        let listener = bind_addr(&config).unwrap();
        assert_eq!(listener.local_addr().unwrap().port(), 2106);
    }
}
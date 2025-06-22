use crate::dto::InboundConnection;
use anyhow::Context;
use log::error;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::task::JoinHandle;
use tracing::info;

pub struct ConnectionListener {
    pub name: String,
    pub cfg: InboundConnection,
}
fn to_ipv4(addr: SocketAddr) -> Option<Ipv4Addr> {
    match addr {
        SocketAddr::V4(v4) => Some(*v4.ip()),
        SocketAddr::V6(_) => None,
    }
}
impl ConnectionListener {
    /// # Errors
    /// - when can't bind ip address and port
    pub fn run<F, Fut>(&self, on_connect: F) -> anyhow::Result<JoinHandle<()>>
    where
        F: FnOnce(TcpStream, Ipv4Addr) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        info!(
            "[{}] Listening on {}:{}",
            self.name, self.cfg.ip, self.cfg.port
        );
        let name = self.name.clone();
        let listener = bind_addr(&self.cfg)?;
        Ok(tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, addr)) => {
                        if let Some(ipv4) = to_ipv4(addr) {
                            info!("[{name}] New connection from {ipv4}");
                            let handler = on_connect.clone();
                            tokio::spawn(handler(stream, ipv4));
                        } else {
                            error!("[{name}] Rejected non-IPv4 connection from {addr}");
                            let _ = stream.shutdown().await; //ignore the error, we don't care about it
                        }
                    }
                    Err(e) => {
                        error!("Error: {e}");
                        break;
                    }
                }
            }
        }))
    }
}

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
    use crate::dto::InboundConnection;
    use crate::network::listener::bind_addr;

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

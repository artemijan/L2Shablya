use crate::common::dto::config::{Connection, Server};
use crate::login_server::controller::Login;
use anyhow::Context;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpSocket};
use crate::database::DBPool;

pub fn create_handle<F, Fut>(
    cfg: Arc<Server>,
    lc: Arc<Login>,
    client_pool: DBPool,
    handler: F,
) -> tokio::task::JoinHandle<()>
where
    F: Fn(Arc<Server>, Arc<Login>, DBPool) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move { handler(cfg, lc, client_pool).await })
}

pub fn bind_addr(config: &Connection) -> anyhow::Result<TcpListener> {
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

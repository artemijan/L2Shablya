use std::sync::Arc;
use sqlx::AnyPool;
use std::future::Future;
use tokio::net::{TcpListener, TcpSocket};
use std::net::ToSocketAddrs;
use anyhow::Context;
use crate::common::dto::config::{Listener, ServerConfig};
use crate::login_server::controller::LoginController;

pub fn create_handle<F, Fut>(
    cfg: Arc<ServerConfig>,
    lc: Arc<LoginController>,
    client_pool: AnyPool,
    handler: F,
) -> tokio::task::JoinHandle<()> where
    F: Fn(Arc<ServerConfig>, Arc<LoginController>, AnyPool) -> Fut + Send + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    tokio::spawn(async move { handler(cfg, lc, client_pool).await })
}

pub async fn bind_addr(config: &Listener) -> anyhow::Result<TcpListener> {
    let addr = format!("{}:{}", &config.ip, &config.port)
        .to_socket_addrs()
        .context(format!("Failed to resolve address {}:{}", config.ip, config.port))?
        .next().context("No address found for the given host and port")?;
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(config.reuse_addr)?;
    socket.set_reuseport(config.reuse_port)?;
    socket.set_nodelay(config.no_delay)?;
    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    Ok(listener)
}
use std::sync::Arc;
use sqlx::AnyPool;
use crate::common::dto::config;
use crate::common::network;
use crate::login_server::controller::Login;
use crate::login_server::traits::PacketHandler;
pub mod traits;
pub mod gs_thread;
pub mod client_thread;
pub mod controller;

pub async fn main_loop<T>(config: Arc<config::Server>, lc: Arc<Login>, pool: AnyPool)
where
    T: PacketHandler + Send + Sync + 'static,
{
    let conn_cfg = T::get_connection_config(&config);
    let listener =
        network::bind_addr(conn_cfg).unwrap_or_else(|_| panic!("Can not bind socket {conn_cfg:?}"));
    println!("{} listening on {}", T::get_handler_name(), &listener.local_addr().unwrap());
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut handler = T::new(stream, pool.clone(), lc.clone());
                tokio::spawn(async move { handler.handle_client().await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {e}");
            }
        }
    }
}
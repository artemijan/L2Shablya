use crate::common::dto::config;
use crate::common::network;
use crate::login_server::controller::Login;
use crate::login_server::gs_handler::GSHandler;
use crate::login_server::PacketHandler;
use sqlx::AnyPool;
use std::sync::Arc;

pub async fn start(config: Arc<config::Server>, lc: Arc<Login>, pool: AnyPool) {
    let listener = network::bind_addr(&config.listeners.game_servers.connection)
        .unwrap_or_else(|_| panic!("Can not bind socket on {:?}", &config.listeners.game_servers));
    println!("Game Servers listening on {}", &listener.local_addr().unwrap());
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut handler = GSHandler::new(stream, pool.clone(), lc.clone());
                tokio::spawn(async move { handler.handle_client().await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {e}");
            }
        }
    }
}

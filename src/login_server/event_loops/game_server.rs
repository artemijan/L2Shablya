use std::sync::Arc;
use sqlx::AnyPool;
use crate::common::dto::config::ServerConfig;
use crate::common::network;
use crate::login_server::controller::LoginController;
use crate::login_server::gs_handler::GSHandler;
use crate::login_server::PacketHandler;

pub async fn start(config: Arc<ServerConfig>, lc: Arc<LoginController>, pool: AnyPool) {
    let listener = network::bind_addr(&config.server.listeners.game_servers)
        .unwrap_or_else(
            |_| panic!("Can not bind socket on {:?}", &config.server.listeners.game_servers)
        );
    println!(
        "Game Servers listening on {}",
        &listener.local_addr().unwrap()
    );
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut handler = GSHandler::new(stream, pool.clone(), lc.clone());
                tokio::spawn(async move { handler.handle_client().await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
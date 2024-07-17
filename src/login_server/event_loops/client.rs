use std::sync::Arc;
use sqlx::AnyPool;
use crate::common::dto::config::ServerConfig;
use crate::common::network;
use crate::login_server::controller::LoginController;
use crate::login_server::ls_handler::ClientHandler;
use crate::login_server::PacketHandler;

pub async fn start(config: Arc<ServerConfig>, lc: Arc<LoginController>, pool: AnyPool) {
    let listener = network::bind_addr(&config.server.listeners.clients)
        .unwrap_or_else(|_| panic!("Can not bind socket {:?}", &config.server.listeners.clients));
    println!("Clients listening on {}", &listener.local_addr().unwrap());
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut handler = ClientHandler::new(
                    stream,
                    pool.clone(),
                    lc.clone(),
                    config.server.client.timeout,
                );
                tokio::spawn(async move { handler.handle_client().await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

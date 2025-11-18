use crate::controller::GameController;
use crate::ls_client::LoginServerClient;
use crate::pl_client::PlayerClient;
use dotenvy::dotenv;
use kameo::Actor;
use l2_core::config::gs::GSServerConfig;
use l2_core::network::connector::Connector;
use l2_core::network::listener::ConnectionListener;
use l2_core::new_db_pool;
use l2_core::traits::ServerConfig;
use l2_core::utils::bootstrap_tokio_runtime;
use sea_orm::sqlx::any::install_default_drivers;
use std::sync::Arc;
use kameo::actor::Spawn;
use tracing::error;

mod controller;
mod cp_factory;
mod ls_client;
mod lsp_factory;
pub mod managers;
mod packets;
mod pl_client;
mod test_utils;

///
/// # Panics
/// - when can't open a socket
/// - when config file not found
/// - when DB is not accessible
/// - when can't run migrations
///
pub fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .init();
    let cfg = Arc::new(GSServerConfig::load("config/game.yaml"));
    install_default_drivers();
    dotenv().ok();

    let runtime = bootstrap_tokio_runtime(cfg.runtime());
    runtime.block_on(async move {
        let pool = new_db_pool(cfg.database()).await;
        let controller = Arc::new(GameController::new(cfg.clone(), &pool).await);
        let clients_listener = ConnectionListener {
            name: "PlayerListener".to_string(),
            cfg: cfg.listeners.clients.connection.clone(),
        };
        let ls_connector = Connector {
            name: "LSListener".to_string(),
            cfg: cfg.listeners.login_server.connection.clone(),
        };
        let cloned_pool = pool.clone();
        let cloned_controller = controller.clone();
        let mut clients_handle = clients_listener
            .run(async move |stream, addr| {
                let (reader, writer) = stream.into_split();
                PlayerClient::spawn((
                    PlayerClient::new(
                        addr,
                        cloned_controller, // Use the cloned values
                        cloned_pool,
                    ),
                    Box::new(reader),
                    Box::new(writer),
                ));
            })
            .expect("Can't start client handler");

        let cloned_controller = controller.clone();
        let cloned_pool = pool.clone();
        let mut ls_handle = ls_connector.run(async move |st, ipv4| {
            let (reader, writer) = st.into_split();
            LoginServerClient::spawn((
                LoginServerClient::new(
                    ipv4,
                    cloned_controller, // Use the cloned values
                    cloned_pool,
                ),
                Box::new(reader),
                Box::new(writer),
            ))
        });
        tokio::select! {
            Err(e) = &mut clients_handle => {
                error!("Client handler exited unexpectedly: {e}");
            }
            Err(e) = &mut ls_handle => {
                error!("Game server handler exited unexpectedly: {e}");
            }
        }
        if !clients_handle.is_finished() {
            clients_handle.abort();
        }
        if !ls_handle.is_finished() {
            ls_handle.abort();
        }
    });
}

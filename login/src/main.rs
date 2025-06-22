mod login_client;

use crate::controller::LoginController;
use crate::gs_client::GameServerClient;
use dotenvy::dotenv;
use kameo::Actor;
use l2_core::config::login::LoginServerConfig;
use l2_core::network::listener::ConnectionListener;
use l2_core::new_db_pool;
use l2_core::traits::ServerConfig;
use l2_core::utils::bootstrap_tokio_runtime;
use login_client::LoginClient;
use sea_orm::sqlx::any::install_default_drivers;
use std::sync::Arc;
use tracing::error;

mod controller;
mod dto;
pub mod enums;
mod gs_client;
mod packet;
mod test_utils;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .init();

    let cfg = Arc::new(LoginServerConfig::load("config/login.yaml"));
    install_default_drivers();
    dotenv().ok();

    let runtime = bootstrap_tokio_runtime(cfg.runtime());
    runtime.block_on(async move {
        let controller = Arc::new(LoginController::new(cfg.clone()));
        let pool = new_db_pool(cfg.database()).await;
        let clients_listener = ConnectionListener {
            name: "PlayerListener".to_string(),
            cfg: cfg.listeners.clients.connection.clone(),
        };
        let gs_listener = ConnectionListener {
            name: "GSListener".to_string(),
            cfg: cfg.listeners.game_servers.connection.clone(),
        };
        let cloned_pool = pool.clone();
        let cloned_controller = controller.clone();
        let mut clients_handle = clients_listener.run(async move |stream, addr| {
            let (reader, writer) = stream.into_split();
            LoginClient::spawn((
                LoginClient::new(
                    addr,
                    cloned_controller, // Use the cloned values
                    cloned_pool,
                ),
                Box::new(reader),
                Box::new(writer),
            ));
        })?;

        let cloned_controller = controller.clone();
        let cloned_pool = pool.clone();
        let mut gs_handle = gs_listener.run(async move |st, ipv4| {
            let (reader, writer) = st.into_split();
            GameServerClient::spawn((
                GameServerClient::new(
                    ipv4,
                    cloned_controller, // Use the cloned values
                    cloned_pool,
                ),
                Box::new(reader),
                Box::new(writer),
            ));
        })?;
        tokio::select! {
            Err(e) = &mut clients_handle => {
                error!("Client handler exited unexpectedly: {e}");
            }
            Err(e) = &mut gs_handle => {
                error!("Game server handler exited unexpectedly: {e}");
            }
        }
        if !clients_handle.is_finished() {
            clients_handle.abort();
        }
        if !gs_handle.is_finished() {
            gs_handle.abort();
        }
        Ok(())
    })
}

use crate::database::new_db_pool;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::controller::Login;
use crate::login_server::gs_thread::GSHandler;
use crate::login_server::traits::PacketHandler;
use anyhow::Context;
use common::network;
use dotenvy::dotenv;
use login_server::dto::config;
use login_server::main_loop;
use sqlx::any::install_default_drivers;
use sqlx::Connection;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::num::NonZero;
use std::sync::Arc;
use std::thread;

mod common;
mod crypt;
mod database;
mod login_server;

///
/// # Panics
///
/// - when Game server thread quits, and so login server can not accept new servers
/// - when client listener thread quits so we can't accept new clients
pub fn main() {
    let config = Arc::new(config::Server::load("config/login.yaml"));
    let mut worker_count = thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    if let Some(wrk_cnt) = &config.runtime {
        worker_count = wrk_cnt.worker_threads;
    }
    println!("Runtime: Worker count {worker_count}");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("login-worker")
        .worker_threads(worker_count)
        .build()
        .expect("Failed to build tokio runtime.");
    rt.block_on(async {
        start(config).await;
    });
}

async fn start(config: Arc<config::Server>) {
    install_default_drivers();
    dotenv().ok();
    let pool = new_db_pool(&config.database).await;
    let lc = Arc::new(Login::new(config.clone()));
    database::run_migrations(&pool).await;
    let clients_handle = network::create_handle(
        config.clone(),
        lc.clone(),
        pool.clone(),
        main_loop::<ClientHandler, config::Server, Login>,
    );

    let gs_handle = network::create_handle(
        config.clone(),
        lc.clone(),
        pool.clone(),
        main_loop::<GSHandler, config::Server, Login>,
    );

    clients_handle
        .await
        .expect("Exiting: Client handler closed");
    // actually this line is never reached, because in previous handle it's infinite loop
    gs_handle.await.expect("Exiting: GS handler closed");
}

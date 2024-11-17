use std::num::NonZero;
use std::sync::Arc;
use std::thread;
use dotenvy::dotenv;
use sqlx::any::install_default_drivers;
use crate::common::network;
use crate::database::new_db_pool;
use crate::game_server::controller::Controller;
use crate::game_server::dto::config::GSServer;

mod game_server;
mod common;
mod crypt;
mod database;

pub fn main() {
    let config = Arc::new(GSServer::load("config/game.yaml"));
    let mut worker_count = thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    if let Some(wrk_cnt) = &config.runtime {
        worker_count = wrk_cnt.worker_threads;
    }
    println!("Runtime: Worker count {worker_count}");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("game-worker")
        .worker_threads(worker_count)
        .build()
        .expect("Failed to build tokio runtime.");
    rt.block_on(async {
        start(config).await;
    });
}


async fn start(config: Arc<GSServer>) {
    install_default_drivers();
    dotenv().ok();
    let pool = new_db_pool(&config.database).await;
    let controller = Arc::new(Controller::new(config.clone()));
    database::run_migrations(&pool).await;
    
    
    // let clients_handle = network::create_handle(
    //     config.clone(),
    //     controller.clone(),
    //     pool.clone(),
    //     main_loop::<ClientHandler>,
    // );
    // 
    // 
    // clients_handle
    //     .await
    //     .expect("Exiting: Client handler closed");
    // actually this line is never reached, because in previous handle it's infinite loop
}
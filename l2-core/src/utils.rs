use crate::dto;
use std::num::NonZero;
use std::thread;
use tokio::runtime::Runtime;
use tracing::info;

/// # Panics
/// - when failed to build tokio runtime
pub fn bootstrap_tokio_runtime(cfg: Option<&dto::Runtime>) -> Runtime {
    let mut worker_count = thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    if let Some(wrk_cnt) = cfg {
        worker_count = wrk_cnt.worker_threads;
    }
    info!("Runtime: Worker count {worker_count}");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("worker")
        .worker_threads(worker_count)
        .build()
        .expect("Failed to build tokio runtime.")
}

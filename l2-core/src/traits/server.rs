use crate::network::bind_addr;
use crate::new_db_pool;
use crate::traits::handlers::{InboundHandler, OutboundHandler, PacketHandler};
use crate::traits::{IpBan, ServerConfig};
use async_trait::async_trait;
use dotenvy::dotenv;
use entities::DBPool;
use sqlx::any::install_default_drivers;
use std::future::Future;
use std::net::IpAddr;
use std::num::NonZero;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tracing::{error, info, instrument};

#[async_trait]
pub trait Server {
    type ConfigType: ServerConfig + Send + Sync + 'static;
    type ControllerType: IpBan + Send + Sync + 'static;
    fn bootstrap<F, Fut>(path: &str, start: F)
    where
        F: Fn(Arc<Self::ConfigType>, DBPool) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let config = Arc::new(Self::ConfigType::load(path));
        let mut worker_count = thread::available_parallelism()
            .map(NonZero::get)
            .unwrap_or(1);
        if let Some(wrk_cnt) = config.runtime() {
            worker_count = wrk_cnt.worker_threads;
        }
        info!("Runtime: Worker count {worker_count}");
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("worker")
            .worker_threads(worker_count)
            .build()
            .expect("Failed to build tokio runtime.");
        install_default_drivers();
        dotenv().ok();
        rt.block_on(async {
            let pool = new_db_pool(config.database()).await;
            start(config, pool).await;
        });
    }

    fn listener_loop<T>(
        config: Arc<Self::ConfigType>,
        controller: Arc<Self::ControllerType>,
        pool: DBPool,
    ) -> JoinHandle<()>
    where
        T: PacketHandler<ConfigType = Self::ConfigType, ControllerType = Self::ControllerType>
            + InboundHandler<ConfigType = Self::ConfigType>
            + Send
            + Sync
            + 'static,
    {
        tokio::spawn(async move {
            let conn_cfg = T::get_connection_config(&config);
            let listener = bind_addr(conn_cfg)
                .unwrap_or_else(|e| panic!("{e}:Can not bind socket {conn_cfg:?}"));
            info!(
                "{} listening on {}",
                T::get_handler_name(),
                &listener
                    .local_addr()
                    .expect("Cannot get socket local address"),
            );
            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        match addr.ip() {
                            IpAddr::V4(ipv4_addr) => {
                                // Skip banned IP addresses
                                if controller.is_ip_banned(&ipv4_addr.to_string()) {
                                    error!("IP is banned, skipping connection from {:?}", addr);
                                    continue;
                                }

                                info!(
                                    "Incoming connection from {:?} ({})",
                                    ipv4_addr,
                                    T::get_handler_name()
                                );

                                let (r, w) = socket.into_split();
                                let mut handler =
                                    T::new(r, w, ipv4_addr, pool.clone(), controller.clone());
                                tokio::spawn(async move {
                                    if let Err(err) = handler.handle_client().await {
                                        error!(
                                            "Closing handler {} with error: {err}",
                                            T::get_handler_name()
                                        );
                                    }
                                });
                            }
                            IpAddr::V6(ip) => {
                                error!("IPv6 connections are not supported, skipping connection from {ip:?}");
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {e}");
                        continue;
                    }
                }
            }
        })
    }
    #[instrument]
    async fn get_stream(address: &str) -> TcpStream {
        loop {
            match TcpStream::connect(address).await {
                Ok(s) => break s, // Exit the loop when connection succeeds
                Err(e) => {
                    error!("Failed to connect: {e}. Retrying in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
    fn connector_loop<T>(
        config: Arc<Self::ConfigType>,
        controller: Arc<Self::ControllerType>,
        pool: DBPool,
    ) -> JoinHandle<()>
    where
        T: PacketHandler<ConfigType = Self::ConfigType, ControllerType = Self::ControllerType>
            + OutboundHandler<ConfigType = Self::ConfigType>
            + Send
            + Sync
            + 'static,
    {
        tokio::spawn(async move {
            let conn_cfg = T::get_connection_config(&config);
            let address = format!("{}:{}", conn_cfg.ip, conn_cfg.port);
            info!("Connecting to {} on: {}", T::get_handler_name(), &address);
            loop {
                let stream = Self::get_stream(&address).await;
                stream
                    .set_nodelay(conn_cfg.no_delay)
                    .expect("Set nodelay failed");
                let IpAddr::V4(ip) = stream.peer_addr().expect("Cannot get peer address").ip()
                else {
                    error!("IP v6 is not supported");
                    break;
                };
                let (r, w) = stream.into_split();
                let mut handler = T::new(r, w, ip, pool.clone(), controller.clone());
                if let Err(err) = handler.handle_client().await {
                    error!(
                        "Closing handler {}, with error: {err}",
                        T::get_handler_name()
                    );
                }
                error!("Lost connection to login server, trying again in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        })
    }
}

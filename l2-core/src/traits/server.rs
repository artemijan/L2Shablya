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
    /// This is a helper function for loading the server configuration from a file, creating DB pool,
    /// and starting the server. It accepts a closure which is needed to create needed 
    /// listeners/connectors. The closure is blocking, once it is finished server exits.
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
    /// This is a helper function for establishing a listener loop and accepting new clients, 
    /// for example listen for connections from game server, or listening for new connections from
    /// players.
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
                    error!("Failed to connect to {address}: {e}. Retrying in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
    ///
    /// This is a helper function for establishing a connection to a remote, and if it fails retry
    /// in N seconds. Currently, it is used for only game server which is trying to connect to 
    /// a login server.   
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
                error!("{}: Lost connection to {ip}, trying again in 5 seconds...", T::get_handler_name());
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::dto::{Database, InboundConnection, Runtime};
    use crate::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
    use crate::traits::server::Server;
    use crate::traits::{IpBan, ServerConfig, Shutdown};
    use anyhow::Error;
    use async_trait::async_trait;
    use entities::DBPool;
    use std::fmt::{Debug, Formatter};
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use std::time::Duration;
    use test_utils::utils::get_test_db;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::{Mutex, Notify};
    use tokio::time::{sleep, timeout};

    struct MockServer;
    struct MockController;

    struct MockConfigType {
        rt: Runtime,
        db: Database,
        inbound: InboundConnection,
    }
    impl MockConfigType {
        fn default() -> Self {
            Self {
                rt: Runtime { worker_threads: 2 },
                db: Database {
                    url: "sqlite::memory:".to_string(),
                    max_connections: 4,
                    min_connections: 2,
                    connect_timeout: 5,
                    idle_timeout: 5,
                    max_lifetime: 60,
                },
                inbound: InboundConnection {
                    ip: "127.0.0.1".to_string(),
                    port: 15999,
                    reuse_addr: true,
                    reuse_port: true,
                    no_delay: true,
                },
            }
        }
    }

    impl ServerConfig for MockConfigType {
        fn load(_: &str) -> Self {
            Self::default()
        }

        fn from_string(_: &str) -> Self {
            Self::default()
        }

        fn runtime(&self) -> Option<&Runtime> {
            Some(&self.rt)
        }

        fn database(&self) -> &Database {
            &self.db
        }
    }
    impl IpBan for MockController {
        fn is_ip_banned(&self, _: &str) -> bool {
            false
        }
    }
    impl Server for MockServer {
        type ConfigType = MockConfigType;
        type ControllerType = MockController;
    }

    struct MockHandler {
        pool: DBPool,
        on_connect_called: bool,
        on_disconnect_called: bool,
        received_bytes: Vec<Vec<u8>>,
        read: Arc<Mutex<dyn AsyncRead + Unpin + Send + 'static>>,
        write: Arc<Mutex<dyn AsyncWrite + Unpin + Send + 'static>>,
    }

    impl MockHandler {
        fn new(
            pool: DBPool,
            read: Arc<Mutex<dyn AsyncRead + Unpin + Send + 'static>>,
            write: Arc<Mutex<dyn AsyncWrite + Unpin + Send + 'static>>,
        ) -> Self {
            MockHandler {
                pool,
                read,
                write,
                received_bytes: Vec::new(),
                on_connect_called: false,
                on_disconnect_called: false,
            }
        }
    }
    #[async_trait]
    impl PacketSender for MockHandler {
        async fn encrypt(&self, _: &mut [u8]) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_encryption_enabled(&self) -> bool {
            false
        }

        async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>> {
            &self.write
        }
    }

    impl Debug for MockHandler {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "MockHandler {{ on_connect_called: {:?}, pool: {:?} }}",
                self.on_connect_called, self.pool
            )
        }
    }

    impl Shutdown for MockHandler {
        fn get_shutdown_listener(&self) -> Arc<Notify> {
            Arc::new(Notify::new())
        }
    }

    #[async_trait]
    impl PacketHandler for MockHandler {
        type ConfigType = MockConfigType;
        type ControllerType = MockController;

        fn get_handler_name() -> &'static str {
            "Test handler"
        }

        fn get_controller(&self) -> &Arc<Self::ControllerType> {
            todo!()
        }

        fn new<R, W>(
            read: R,
            write: W,
            _: Ipv4Addr,
            pool: DBPool,
            _: Arc<Self::ControllerType>,
        ) -> Self
        where
            R: AsyncRead + Unpin + Send + 'static,
            W: AsyncWrite + Unpin + Send + 'static,
        {
            MockHandler::new(
                pool,
                Arc::new(Mutex::new(read)),
                Arc::new(Mutex::new(write)),
            )
        }

        async fn on_connect(&mut self) -> anyhow::Result<()> {
            self.on_connect_called = true;
            self.write
                .lock()
                .await
                .write_all(&[1, 1, 1, 0, 1, 1, 1])
                .await?;
            Ok(())
        }

        async fn on_disconnect(&mut self) {
            self.on_disconnect_called = true;
        }

        fn get_stream_reader_mut(&self) -> &Arc<Mutex<dyn AsyncRead + Send + Unpin>> {
            &self.read
        }

        fn get_timeout(&self) -> Option<u64> {
            None
        }

        fn get_db_pool(&self) -> &DBPool {
            &self.pool
        }

        async fn on_receive_bytes(&mut self, _: usize, bytes: &mut [u8]) -> Result<(), Error> {
            self.received_bytes.push(bytes.to_vec());
            Ok(())
        }
    }

    impl InboundHandler for MockHandler {
        type ConfigType = MockConfigType;

        fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
            &cfg.inbound
        }
    }
    #[test]
    fn test_bootstrap() {
        MockServer::bootstrap("", |cfg, pool| async move {
            assert_eq!(cfg.db.max_connections, 4);
            assert_eq!(cfg.rt.worker_threads, 2);
            assert!(pool.ping().await.is_ok());
        });
    }

    #[test]
    fn test_listener() {
        //this is just a simple check if we can bind to local host on port 2106
        // after it's bind, we just simply abort the task.
        MockServer::bootstrap("", |cfg, pool| async move {
            let l_loop =
                MockServer::listener_loop::<MockHandler>(cfg, Arc::new(MockController), pool);
            let result = timeout(Duration::from_secs(1), l_loop).await;

            if let Err(e) = result {
                assert_eq!(e.to_string(), "deadline has elapsed");
            }
        });
    }
    #[tokio::test]
    async fn test_loop() {
        let cfg = Arc::new(MockConfigType::load(""));
        let pool = get_test_db().await;
        let l_loop = MockServer::listener_loop::<MockHandler>(cfg, Arc::new(MockController), pool);
        let handle = tokio::spawn(async {
            l_loop.await.unwrap();
        });
        sleep(Duration::from_secs(2)).await; //give it a time to establish socket listener
        let mut stream = TcpStream::connect("127.0.0.1:15999")
            .await
            .expect("Failed to connect");
        // Receive a response from the server
        let mut buffer = [0; 7];
        stream
            .read_exact(&mut buffer)
            .await
            .expect("Failed to read from stream");
        assert_eq!([1, 1, 1, 0, 1, 1, 1], buffer);
        handle.abort();
    }
}

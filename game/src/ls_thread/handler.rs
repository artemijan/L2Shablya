use crate::controller::Controller;
use crate::lsp_factory::build_ls_packet;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::crypt::login::Encryption;
use l2_core::dto::OutboundConnection;
use l2_core::traits::handlers::{OutboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use tracing::{info, instrument};

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions, unused)]
pub struct LoginHandler {
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    db_pool: DBPool,
    controller: Arc<Controller>,
    shutdown_notifier: Arc<Notify>,
    timeout: u8,
    blowfish: Encryption,
}
impl LoginHandler {
    pub const HANDLER_ID: u8 = 1;
    pub fn set_blowfish(&mut self, blowfish: Encryption) {
        self.blowfish = blowfish;
    }
}
impl Shutdown for LoginHandler {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_notifier.clone()
    }
}

#[async_trait]
impl PacketSender for LoginHandler {
    fn encryption(&self) -> Option<&Encryption> {
        Some(&self.blowfish)
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }
}

#[async_trait::async_trait]
impl PacketHandler for LoginHandler {
    type ConfigType = GSServer;
    type ControllerType = Controller;

    fn get_handler_name() -> &'static str {
        "Login handler"
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.controller
    }

    fn new(stream: TcpStream, db_pool: DBPool, controller: Arc<Self::ControllerType>) -> Self {
        let (tcp_reader, tcp_writer) = stream.into_split();
        let cfg = controller.get_cfg();
        Self {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            shutdown_notifier: Arc::new(Notify::new()),
            controller,
            db_pool,
            blowfish: Encryption::from_u8_key(cfg.blowfish_key.as_bytes()),
            timeout: cfg.client.timeout,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> anyhow::Result<()> {
        info!("Connected to Login server. Trying to Authenticate.");
        Ok(())
    }

    async fn on_disconnect(&mut self){
        self.controller.message_broker.unregister_packet_handler(Self::HANDLER_ID);
        info!("Login server disconnected");
    }

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    fn get_timeout(&self) -> Option<u64> {
        None
    }

    fn get_db_pool(&self) -> &DBPool {
        &self.db_pool
    }

    #[instrument(skip(self, bytes))]
    async fn on_receive_bytes(&mut self, _: usize, bytes: &mut [u8]) -> Result<(), Error> {
        self.blowfish.decrypt(bytes)?;
        if !Encryption::verify_checksum(bytes) {
            bail!("Can not verify check sum.");
        }
        let handler = build_ls_packet(bytes)?;
        handler.handle(self).await
    }
}

impl OutboundHandler for LoginHandler {
    type ConfigType = GSServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &OutboundConnection {
        &cfg.listeners.login_server.connection
    }
}

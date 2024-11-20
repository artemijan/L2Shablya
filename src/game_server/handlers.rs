use crate::common::dto::{InboundConnection, OutboundConnection};
use crate::common::errors::Packet;
use crate::common::packets::error::PacketRun;
use crate::common::packets::SendablePacket;
use crate::common::traits::handlers::{OutboundHandler, PacketHandler};
use crate::common::traits::Shutdown;
use crate::database::DBPool;
use crate::game_server::controller::Controller;
use crate::game_server::dto::config::GSServer;
use anyhow::Error;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};

pub struct PlayerHandler;
pub struct LoginHandler{
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    db_pool: DBPool,
    controller: Arc<Controller>,
    shutdown_notifier: Arc<Notify>,
    timeout: u8,
    connection_start_time: SystemTime,
}

impl Shutdown for LoginHandler {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_notifier.clone()
    }
}

#[async_trait::async_trait]
impl PacketHandler for LoginHandler {
    type ConfigType = GSServer;
    type ControllerType = Controller;

    fn get_handler_name() -> String {
        "Player handler".to_string()
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        todo!()
    }

    fn new(stream: TcpStream, db_pool: DBPool, controller: Arc<Self::ControllerType>) -> Self {
        let connection_start_time = SystemTime::now();
        let (tcp_reader, tcp_writer) = stream.into_split();
        let cfg = controller.get_cfg();
        Self {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            shutdown_notifier: Arc::new(Notify::new()),
            controller,
            db_pool,
            timeout: cfg.client.timeout,
            connection_start_time,
        }
    }

    async fn on_connect(&mut self) -> Result<(), Packet> {
        println!("Connected to Login server. Trying to Authenticate.");
        Ok(())
    }

    fn on_disconnect(&mut self) {
        todo!()
    }

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        Some(u64::from(self.timeout))
    }

    async fn send_packet(
        &self,
        packet: Box<dyn SendablePacket>,
    ) -> Result<Box<dyn SendablePacket>, Error> {
        todo!()
    }

    async fn send_bytes(&self, bytes: &mut [u8]) -> Result<(), Error> {
        todo!()
    }

    fn get_db_pool_mut(&mut self) -> &mut DBPool {
        todo!()
    }

    async fn on_receive_bytes(
        &mut self,
        packet_size: usize,
        bytes: &mut [u8],
    ) -> Result<(), Error> {
        println!("Packet in with size {packet_size}, body: {bytes:?}");
        Ok(())
    }

    async fn handle_result(
        &mut self,
        resp: Result<Option<Box<dyn SendablePacket>>, PacketRun>,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl OutboundHandler for LoginHandler {
    type ConfigType = GSServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &OutboundConnection {
        &cfg.listeners.login_server.connection
    }
}

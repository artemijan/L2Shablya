use crate::common::dto::Connection;
use crate::common::errors::Packet;
use crate::common::packet::error::PacketRun;
use crate::common::packet::SendablePacket;
use crate::common::traits::Shutdown;
use crate::database::DBPool;
use crate::game_server::controller::Controller;
use crate::game_server::dto::config::GSServer;
use anyhow::Error;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use crate::common::traits::handler::PacketHandler;

pub struct PlayerHandler;

impl Shutdown for PlayerHandler {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        todo!()
    }
}

#[async_trait::async_trait]
impl PacketHandler for PlayerHandler {
    type ConfigType = GSServer;
    type ControllerType = Controller;

    fn get_handler_name() -> String {
        "Player handler".to_string()
    }

    fn get_connection_config(cfg: &Self::ConfigType) -> &Connection {
        &cfg.listeners.clients.connection
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        todo!()
    }

    fn new(stream: TcpStream, db_pool: DBPool, lc: Arc<Self::ControllerType>) -> Self {
        todo!()
    }

    async fn on_connect(&mut self) -> Result<(), Packet> {
        todo!()
    }

    fn on_disconnect(&mut self) {
        todo!()
    }

    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>> {
        todo!()
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        todo!()
    }

    fn get_timeout(&self) -> Option<u64> {
        todo!()
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
        todo!()
    }

    async fn read_packet(&mut self) -> anyhow::Result<(usize, Vec<u8>)> {
        todo!()
    }

    async fn handle_result(
        &mut self,
        resp: Result<Option<Box<dyn SendablePacket>>, PacketRun>,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn handle_client(&mut self) {
        todo!()
    }
}

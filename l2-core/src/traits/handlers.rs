use crate::crypt::login::Encryption;
use crate::dto;
use crate::errors::Packet;
use crate::shared_packets::common::SendablePacket;
use crate::traits::Shutdown;
use anyhow::Error;
use async_trait::async_trait;
use entities::DBPool;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, instrument};

pub const PACKET_SIZE_BYTES: usize = 2;

pub trait InboundHandler {
    type ConfigType;
    fn get_connection_config(cfg: &Self::ConfigType) -> &dto::InboundConnection;
}

pub trait OutboundHandler {
    type ConfigType;
    fn get_connection_config(cfg: &Self::ConfigType) -> &dto::OutboundConnection;
}
#[async_trait]
pub trait PacketSender: Send + Sync + Debug {
    ///
    /// # Errors
    /// - when packet is too large
    fn add_padding(&self, packet: &mut Box<dyn SendablePacket>) -> Result<(), Error> {
        let buffer = packet.get_buffer_mut();
        buffer.write_i32(0)?;
        let padding = (buffer.get_size() - 2) % 8;
        if padding != 0 {
            for _ in padding..8 {
                buffer.write_u8(0)?;
            }
        }
        Ok(())
    }
    async fn send_packet(&self, mut packet: Box<dyn SendablePacket>) -> Result<(), Error> {
        if self.encryption().is_some() {
            self.add_padding(&mut packet)?;
            let bytes = packet.get_bytes_mut();
            self.send_bytes(bytes).await?;
            Ok(())
        } else {
            let bytes = packet.get_bytes_mut();
            self.send_bytes(bytes).await?;
            Ok(())
        }
    }
    async fn send_bytes(&self, bytes: &mut [u8]) -> Result<(), Error> {
        let size = bytes.len();
        if let Some(blowfish) = self.encryption() {
            Encryption::append_checksum(&mut bytes[2..size]);
            blowfish.encrypt(&mut bytes[2..size]);
        }
        self.get_stream_writer_mut()
            .await
            .lock()
            .await
            .write_all(bytes)
            .await?;
        Ok(())
    }
    fn encryption(&self) -> Option<&Encryption>;
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>>;
}
#[async_trait]
pub trait PacketHandler: PacketSender + Shutdown + Send + Sync + Debug {
    type ConfigType;
    type ControllerType;

    fn get_handler_name() -> &'static str;
    fn get_controller(&self) -> &Arc<Self::ControllerType>;
    fn new(stream: TcpStream, db_pool: DBPool, lc: Arc<Self::ControllerType>) -> Self;

    async fn on_connect(&mut self) -> Result<(), Packet>;
    async fn on_disconnect(&mut self);
    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>>;

    fn get_timeout(&self) -> Option<u64>;

    fn get_db_pool(&self) -> &DBPool;

    async fn on_receive_bytes(&mut self, packet_size: usize, bytes: &mut [u8])
        -> Result<(), Error>;

    #[instrument(skip(self))]
    async fn read_packet(&mut self) -> anyhow::Result<(usize, Vec<u8>)> {
        let mut size_buf = [0; PACKET_SIZE_BYTES];
        let mut socket = self.get_stream_reader_mut().lock().await;
        if socket.read_exact(&mut size_buf).await.is_err() {
            // at this stage, client wanted to disconnect
            return Ok((0, vec![]));
        }
        let size = (u16::from_le_bytes(size_buf) as usize) - PACKET_SIZE_BYTES;
        // Read the body of the packet based on the size
        let mut body = vec![0; size];
        socket.read_exact(&mut body).await?;
        Ok((size, body))
    }

    #[instrument(skip(self))]
    async fn handle_client(&mut self) {
        let client_addr = self
            .get_stream_reader_mut()
            .lock()
            .await
            .peer_addr()
            .unwrap();
        if let Err(e) = self.on_connect().await {
            error!(
                "{}: Disconnecting client. Error: {}",
                Self::get_handler_name(),
                e
            );
            self.on_disconnect().await;
            return;
        }
        let shutdown_listener = self.get_shutdown_listener(); //shutdown listener must be cloned only once before the loop
        loop {
            let timeout_future = if let Some(t_out) = self.get_timeout() {
                sleep(Duration::from_secs(t_out))
            } else {
                sleep(Duration::MAX) // Use a long sleep for no timeout
            };
            let read_future = self.read_packet();
            tokio::select! {
                read_result = read_future =>{
                    match read_result {
                        Ok((0, _)) => {
                            self.on_disconnect().await;
                            break;
                        }
                        Ok((bytes_read, mut data)) => {
                            if let Err(e) = self.on_receive_bytes(bytes_read, &mut data).await {
                                error!(
                                    "{}: Disconnecting client {}, because error occurred {}",
                                    Self::get_handler_name(),
                                    client_addr,
                                    e
                                );
                                self.on_disconnect().await;
                                break;
                            }
                        }
                        Err(e) => {
                            error!("{}: Failed to read data from client: {}", Self::get_handler_name(), e);
                            self.on_disconnect().await;
                            break;
                        }
                    }
                }
                // Handle timeout event separately
                () = timeout_future => {
                    info!(
                        "{}: No data received within timeout. Dropping connection.",
                        Self::get_handler_name()
                    );
                    self.on_disconnect().await;
                    break;
                }
                // Handle shutdown notification (or other task notifications)
                () = shutdown_listener.notified() => {
                    info!("{}: Received shutdown notification. Dropping connection.", Self::get_handler_name());
                    self.on_disconnect().await;
                    break;
                }
            }
        }
    }
}

use crate::dto;
use crate::shared_packets::common::SendablePacket;
use crate::traits::Shutdown;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::DBPool;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{info, instrument};

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
    async fn send_packet(&self, mut packet: Box<dyn SendablePacket>) -> Result<(), Error> {
        self.send_bytes(packet.get_bytes(self.is_encryption_enabled()))
            .await?;
        Ok(())
    }
    async fn send_bytes(&self, bytes: &mut [u8]) -> Result<(), Error> {
        if self.is_encryption_enabled() {
            self.encrypt(bytes).await?;
        }
        self.get_stream_writer_mut()
            .await
            .lock()
            .await
            .write_all(bytes)
            .await?;
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    async fn encrypt(&self, bytes: &mut [u8]) -> anyhow::Result<()>;
    fn is_encryption_enabled(&self) -> bool;
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>>;
}
#[async_trait]
pub trait PacketHandler: PacketSender + Shutdown + Send + Sync + Debug {
    type ConfigType;
    type ControllerType;

    fn get_handler_name() -> &'static str;
    fn get_controller(&self) -> &Arc<Self::ControllerType>;
    fn new<R, W>(
        read: R,
        write: W,
        ip: Ipv4Addr,
        db_pool: DBPool,
        lc: Arc<Self::ControllerType>,
    ) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static;

    async fn on_connect(&mut self) -> anyhow::Result<()>;
    async fn on_disconnect(&mut self);
    fn get_stream_reader_mut(&self) -> &Arc<Mutex<dyn AsyncRead + Send + Unpin>>;

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
    async fn handle_client(&mut self) -> anyhow::Result<()> {
        self.on_connect().await?;
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
                            // it's okay that we received 0 bytes, client wants to close socket
                            break;
                        }
                        Ok((bytes_read, mut data)) => {
                            if let Err(e) = self.on_receive_bytes(bytes_read, &mut data).await {
                                self.on_disconnect().await;
                                bail!("Error receiving packet: {}", e);
                            }
                        }
                        Err(e) => {
                            self.on_disconnect().await;
                            bail!("Error receiving packet: {}", e);
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
        Ok(())
    }
}

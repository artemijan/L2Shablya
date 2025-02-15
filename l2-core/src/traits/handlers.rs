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
        let defined_size = u16::from_le_bytes(size_buf) as usize;
        if defined_size == 0 {
            bail!("Packet size is 0, it's not expected");
        }
        let size = defined_size - PACKET_SIZE_BYTES;
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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Error;
    use async_trait::async_trait;
    use entities::DBPool;
    use ntest::timeout;
    use core::time;
    use std::fmt::Debug;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncRead, AsyncWrite};
    use tokio::sync::Mutex;

    struct TestController;

    struct TestPacketHandler {
        db_pool: DBPool,
        on_receive_bytes: Vec<(usize, Vec<u8>)>,
        controller: Arc<TestController>,
        on_connect_called: u8,
        on_disconnect_called: u8,
        reader: Arc<Mutex<dyn AsyncRead + Send + Unpin>>,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    }

    impl Debug for TestPacketHandler {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestPacketHandler")
                .field("on_receive_bytes", &self.on_receive_bytes)
                .field("on_connect_called", &self.on_connect_called)
                .field("on_disconnect_called", &self.on_disconnect_called)
                .finish_non_exhaustive()
        }
    }

    #[async_trait]
    impl PacketHandler for TestPacketHandler {
        type ConfigType = ();
        type ControllerType = TestController;

        fn get_handler_name() -> &'static str {
            "TestPacketHandler"
        }
        fn get_timeout(&self) -> Option<u64> {
            Some(5)
        }

        fn get_db_pool(&self) -> &DBPool {
            &self.db_pool
        }
        async fn on_receive_bytes(
            &mut self,
            packet_size: usize,
            bytes: &mut [u8],
        ) -> Result<(), Error> {
            self.on_receive_bytes.push((packet_size, bytes.to_vec()));
            Ok(())
        }

        fn get_controller(&self) -> &Arc<Self::ControllerType> {
            &self.controller
        }

        fn new<R, W>(
            read: R,
            write: W,
            _ip: Ipv4Addr,
            db_pool: DBPool,
            lc: Arc<Self::ControllerType>,
        ) -> Self
        where
            R: AsyncRead + Unpin + Send + 'static,
            W: AsyncWrite + Unpin + Send + 'static,
        {
            Self {
                db_pool,
                on_receive_bytes: vec![],
                controller: lc,
                on_connect_called: 0,
                on_disconnect_called: 0,
                reader: Arc::new(Mutex::new(read)),
                writer: Arc::new(Mutex::new(write)),
            }
        }

        async fn on_connect(&mut self) -> anyhow::Result<()> {
            self.on_connect_called += 1;
            Ok(())
        }

        async fn on_disconnect(&mut self) {
            self.on_disconnect_called += 1;
        }

        fn get_stream_reader_mut(&self) -> &Arc<Mutex<dyn AsyncRead + Send + Unpin>> {
            &self.reader
        }
    }
    impl Shutdown for TestPacketHandler {
        fn get_shutdown_listener(&self) -> Arc<tokio::sync::Notify> {
            Arc::new(tokio::sync::Notify::new())
        }
        fn shutdown(&self) {}
    }

    #[async_trait]
    impl PacketSender for TestPacketHandler {
        async fn encrypt(&self, bytes: &mut [u8]) -> anyhow::Result<()> {
            bytes[0] = 1;
            bytes[1] = 2;
            bytes[2] = 3;
            Ok(())
        }

        fn is_encryption_enabled(&self) -> bool {
            false
        }

        async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>> {
            &self.writer
        }
    }
    #[tokio::test]
    #[timeout(2000)]
    async fn test_handle_client() {
        let (mut client, server) = tokio::io::duplex(1024);
        let db_pool = get_test_db().await;
        let controller = Arc::new(TestController);
        let (read, write) = split(server);
        let mut handler = TestPacketHandler::new(
            read,
            write,
            Ipv4Addr::new(127, 0, 0, 1),
            db_pool.clone(),
            controller.clone(),
        );
        client.write_all(&[0, 0, 0, 0, 0, 0, 0, 0]).await.unwrap();
        let err = handler.handle_client().await;
        // assert!(handler.on_connect_called == 1);
        // assert!(handler.on_disconnect_called == 1);
        assert!(err.is_err());
        assert!(
            matches!(err, Err(e) if e.to_string() == "Error receiving packet: Packet size is 0, it's not expected")
        );
    }
    #[tokio::test]
    #[timeout(2000)]
    async fn test_handle_client_receive_bytes() {
        let (mut client, server) = tokio::io::duplex(1024);
        let db_pool = get_test_db().await;
        let controller = Arc::new(TestController);
        let (read, write) = split(server);
        let mut handler = TestPacketHandler::new(
            read,
            write,
            Ipv4Addr::new(127, 0, 0, 1),
            db_pool.clone(),
            controller.clone(),
        );
        client.write_all(&[8, 0, 0, 0, 0, 0, 0, 0]).await.unwrap();
        let handle = tokio::spawn( async move {
            handler.handle_client().await.unwrap();
            handler
        });
        tokio::time::sleep(time::Duration::from_millis(500)).await;
        client.shutdown().await.unwrap();
        let handler = handle.await.unwrap();
        assert!(handler.on_connect_called == 1);
        assert!(handler.on_disconnect_called == 1);
        assert_eq!(handler.on_receive_bytes, vec![(6, vec![0, 0, 0, 0, 0, 0])]);
    }
}

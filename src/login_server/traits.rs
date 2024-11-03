use crate::common::dto::config::{Connection, Server};
use crate::common::errors::Packet;
use crate::common::errors::Packet::UnableToHandleClient;
use crate::login_server::controller::Login;
use crate::packet::common::SendablePacket;
use crate::packet::error::PacketRun;
use anyhow::{bail, Error};
use async_trait::async_trait;
use sqlx::AnyPool;
use tokio::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;


pub trait Shutdown {
    fn get_shutdown_listener(&self) -> Arc<Notify>;
    fn shutdown(&self) {
        self.get_shutdown_listener().notify_one();
    }
}


pub const PACKET_SIZE_BYTES: usize = 2;

#[async_trait]
pub trait PacketHandler: Shutdown {
    fn get_handler_name() -> String;
    fn get_connection_config(cfg: &Server) -> &Connection;
    fn get_lc(&self) -> &Arc<Login>;
    fn new(stream: TcpStream, db_pool: AnyPool, lc: Arc<Login>) -> Self;

    async fn on_connect(&mut self) -> Result<(), Packet>;
    async fn on_disconnect(&mut self);
    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>>;
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>>;
    fn get_timeout(&self) -> Option<u64>;

    async fn send_packet(&self, packet: Box<dyn SendablePacket>) -> Result<Box<dyn SendablePacket>, Error>;

    async fn send_bytes(&self, bytes: &mut [u8]) -> Result<(), Error>;
    async fn on_receive_bytes(
        &mut self,
        packet_size: usize,
        bytes: &mut [u8],
    ) -> Result<(), Error>;

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
    async fn handle_result(
        &mut self,
        resp: Result<Option<Box<dyn SendablePacket>>, PacketRun>,
    ) -> Result<(), Error> {
        match resp {
            Ok(result) => {
                if let Some(packet) = result {
                    self.send_packet(packet).await?;
                }
            }
            Err(e) => {
                if let Some(packet) = e.response {
                    self.send_packet(packet).await?;
                } else if let Some(msg) = e.msg {
                    bail!(UnableToHandleClient { msg })
                }
            }
        }
        Ok(())
    }

    async fn handle_client(&mut self) {
        let client_addr = self
            .get_stream_reader_mut()
            .lock()
            .await
            .peer_addr()
            .unwrap();
        if let Err(e) = self.on_connect().await {
            eprintln!(
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
                                eprintln!(
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
                            eprintln!("{}: Failed to read data from client: {}", Self::get_handler_name(), e);
                            self.on_disconnect().await;
                            break;
                        }
                    }
                }
                // Handle timeout event separately
                () = timeout_future => {
                    println!(
                        "{}: No data received within timeout. Dropping connection.",
                        Self::get_handler_name()
                    );
                    self.on_disconnect().await;
                    break;
                }
                // Handle shutdown notification (or other task notifications)
                () = shutdown_listener.notified() => {
                    println!("{}: Received shutdown notification. Dropping connection.", Self::get_handler_name());
                    self.on_disconnect().await;
                    break;
                }
            }
        }
    }
}

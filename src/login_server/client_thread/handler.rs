use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use sqlx::AnyPool;
use std::time::SystemTime;
use async_trait::async_trait;
use tokio::net::{TcpSocket, TcpStream};
use anyhow::{Context, Error};
use rand::Rng;
use tokio::io::AsyncWriteExt;
use tokio::net::unix::SocketAddr;
use crate::common::dto::config::{Connection, Server};
use crate::common::errors::Packet;
use crate::common::errors::Packet::{ClientPacketNotFound, UnableToSendInit};
use crate::common::session::SessionKey;
use crate::crypt::generate_blowfish_key;
use crate::crypt::login::Encryption;
use crate::crypt::rsa::ScrambledRSAKeyPair;
use crate::login_server::controller::Login;
use crate::login_server::traits::{PacketHandler, Shutdown};
use crate::packet::common::{ClientHandle, SendablePacket};
use crate::packet::ls_factory::build_client_packet;
use crate::packet::to_client::Init;

#[derive(Clone, Debug)]
pub struct Client {
    blowfish_key: Vec<u8>,
    pub ip: Ipv4Addr,
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    pub account_name: Option<String>,
    db_pool: AnyPool,
    session_id: i32,
    shutdown_notifier: Arc<Notify>,
    pub lc: Arc<Login>,
    encryption: Encryption,
    timeout: u8,
    session_key: SessionKey,
    connection_start_time: SystemTime,
    rsa_key_pair: ScrambledRSAKeyPair,
}

/// # This function called each time when there is a new connection.
/// ## Arguments
/// - stream - `TcpStream` to send/receive packets
/// - lc - `LoginClient` object is needed to store some client related data
///
/// # The whole login process looks like the following:
/// 1. send init ()
/// 2. wait request `AuthGG`
/// 3. `AuthGG::new(session_id);` // send `AuthGG` OK
/// 4. wait request auth login
/// 5. `LoginOk::new(session_key);` //send `AuthOk`
/// 6. wait for `RequestServerList`
/// 7. Respond with `ServerList::new()`
///
impl Client {
    pub fn get_session_id(&self) -> i32 {
        self.session_id
    }
    pub fn get_db_pool_mut(&mut self) -> &mut AnyPool {
        &mut self.db_pool
    }
    pub fn get_session_key(&self) -> &SessionKey {
        &self.session_key
    }
    fn get_ipv4_from_socket(socket: &TcpStream) -> Ipv4Addr {
        let default = Ipv4Addr::new(127, 0, 0, 1);
        match socket.peer_addr() {
            Ok(addr) => {
                match addr.ip() {
                    IpAddr::V4(ipv4) => ipv4,
                    IpAddr::V6(_) => default,
                }
            }
            _ => default,
        }
    }
}

impl Shutdown for Client {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_notifier.clone()
    }

    fn shutdown(&self) {
        self.shutdown_notifier.notify_one();
    }
}

#[async_trait]
impl PacketHandler for Client {
    fn get_handler_name() -> String {
        "Login client handler".to_string()
    }
    fn get_connection_config(cfg: &Server) -> &Connection {
        &cfg.listeners.clients.connection
    }
    fn get_lc(&self) -> &Arc<Login> {
        &self.lc
    }

    fn new(stream: TcpStream, db_pool: AnyPool, lc: Arc<Login>) -> Self {
        let mut rng = rand::thread_rng();
        let blowfish_key = generate_blowfish_key();
        let encryption = Encryption::new(&blowfish_key.clone());
        let session_id = rng.gen();
        let connection_start_time = SystemTime::now();
        let timeout = lc.get_config().client.timeout;
        let ip = Self::get_ipv4_from_socket(&stream);
        let (tcp_reader, tcp_writer) = stream.into_split();
        Client {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            db_pool,
            ip,
            shutdown_notifier: Arc::new(Notify::new()),
            encryption,
            session_id,
            session_key: SessionKey::new(),
            account_name: None,
            connection_start_time,
            rsa_key_pair: lc.get_random_rsa_key_pair(),
            blowfish_key: blowfish_key.to_vec(),
            timeout,
            lc,
        }
    }

    async fn on_connect(&mut self) -> Result<(), Packet> {
        let addr = self.tcp_reader.lock().await.peer_addr().unwrap();
        println!("Client connected: {:?}", addr);
        let mut init = Init::new(
            self.session_id,
            self.rsa_key_pair.get_scrambled_modulus(),
            self.blowfish_key.clone(),
        );
        self.tcp_writer
            .lock()
            .await
            .write_all(init.buffer.get_data_mut())
            .await
            .map_err(|_| UnableToSendInit)?;
        Ok(())
    }

    async fn on_disconnect(&mut self) {
        //todo: think of how we cna handle client disconnection
        println!("Player disconnected: {:?}", self.session_id);
    }
    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        Some(u64::from(self.timeout))
    }

    async fn send_packet(&self, mut packet: Box<dyn SendablePacket>) -> Result<Box<dyn SendablePacket>, Error> {
        self.send_bytes(packet.get_bytes_mut()).await?;
        Ok(packet)
    }

    async fn send_bytes(&self, bytes: &mut [u8]) -> Result<(), Error> {
        self.tcp_writer
            .lock()
            .await
            .write_all(bytes)
            .await
            .with_context(|| "Failed to flush packet to socket")
    }

    async fn on_receive_bytes(&mut self, _: usize, data: &mut [u8]) -> Result<(), Error> {
        self.encryption.decrypt(data)?;
        let handler =
            build_client_packet(data, &self.rsa_key_pair).ok_or_else(|| ClientPacketNotFound {
                opcode: data[0] as usize,
            })?;
        let resp = handler.handle(self).await;
        self.handle_result(resp).await
    }
}
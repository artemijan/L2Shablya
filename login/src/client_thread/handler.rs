use crate::controller::Login;
use crate::packet::cp_factory::build_client_packet;
use crate::packet::to_client::Init;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::DBPool;
use l2_core::config::login::LoginServer;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::{generate_blowfish_key, rsa};
use l2_core::dto::InboundConnection;
use l2_core::errors::Packet;
use l2_core::session::SessionKey;
use l2_core::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use tracing::{error, info, instrument};

#[derive(Clone, Debug)]
pub struct Client {
    blowfish_key: Vec<u8>,
    pub ip: Ipv4Addr,
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    pub account_name: Option<String>,
    db_pool: DBPool,
    session_id: i32,
    shutdown_notifier: Arc<Notify>,
    pub lc: Arc<Login>, // login controller
    encryption: Encryption,
    timeout: u8,
    session_key: SessionKey,
    rsa_key_pair: rsa::ScrambledRSAKeyPair,
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
    pub fn check_session(&self, s_key1: i32, s_key2: i32) -> anyhow::Result<()> {
        if !self.session_key.check_session(s_key1, s_key2) {
            bail!("Session key check failed");
        }
        Ok(())
    }
    pub fn get_session_id(&self) -> i32 {
        self.session_id
    }
    pub fn get_session_key(&self) -> &SessionKey {
        &self.session_key
    }
    fn get_ipv4_from_socket(socket: &TcpStream) -> Ipv4Addr {
        let default = Ipv4Addr::new(127, 0, 0, 1);
        match socket.peer_addr() {
            Ok(addr) => match addr.ip() {
                IpAddr::V4(ipv4) => ipv4,
                IpAddr::V6(_) => default,
            },
            _ => default,
        }
    }
}

impl Shutdown for Client {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_notifier.clone()
    }
}

#[async_trait]
impl PacketHandler for Client {
    type ConfigType = LoginServer;
    type ControllerType = Login;
    fn get_handler_name() -> &'static str {
        "Login client handler"
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.lc
    }

    fn new(stream: TcpStream, db_pool: DBPool, lc: Arc<Self::ControllerType>) -> Self {
        let blowfish_key = generate_blowfish_key(None);
        let encryption = Encryption::new(&blowfish_key.clone());
        let session_id = Login::generate_session_id();
        let timeout = lc.get_config().client.timeout;
        let ip = Self::get_ipv4_from_socket(&stream);
        let (tcp_reader, tcp_writer) = stream.into_split();
        Self {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            db_pool,
            ip,
            shutdown_notifier: Arc::new(Notify::new()),
            encryption,
            session_id,
            session_key: SessionKey::new(),
            account_name: None,
            rsa_key_pair: lc.get_random_rsa_key_pair(),
            blowfish_key,
            timeout,
            lc,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> Result<(), Packet> {
        let addr = self.tcp_reader.lock().await.peer_addr().unwrap();
        info!("Client connected: {:?}", addr);
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
            .map_err(|_| Packet::UnableToSendInit)?;
        Ok(())
    }

    fn on_disconnect(&mut self) {
        info!("Player disconnected: {:?}", self.session_id);
    }
    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    fn get_timeout(&self) -> Option<u64> {
        Some(u64::from(self.timeout))
    }

    fn get_db_pool_mut(&mut self) -> &mut DBPool {
        &mut self.db_pool
    }

    #[instrument(skip(self, data))]
    async fn on_receive_bytes(&mut self, _: usize, data: &mut [u8]) -> Result<(), Error> {
        self.encryption.decrypt(data)?;
        let handler = build_client_packet(data, &self.rsa_key_pair).ok_or_else(|| {
            Packet::ClientPacketNotFound {
                opcode: data[0] as usize,
            }
        })?;
        if let Err(err) = handler.handle(self).await {
            error!("Client error: {err:?}");
            return Err(err.into());
        };
        Ok(())
    }

    
}

#[async_trait]
impl PacketSender for Client {
    fn encryption(&self) -> Option<&Encryption> {
        None
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }
}

impl InboundHandler for Client {
    type ConfigType = LoginServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
        &cfg.listeners.clients.connection
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use l2_core::config::login::LoginServer;
    use l2_core::tests::get_test_db;
    use l2_core::traits::ServerConfig;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_init_packet_sent() {
        // Create a listener on a local port
        let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let db_pool = get_test_db().await;
        let cfg = LoginServer::from_string(include_str!("../test_data/test_config.yaml"));
        let lc = Arc::new(Login::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        // Spawn a server task to handle a single connection
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut ch = Client::new(socket, db_pool, cloned_lc);
            ch.handle_client().await;
        });
        // Create a client to connect to the server
        let mut client = TcpStream::connect(addr).await.unwrap();
        // Read the response from the server
        let mut init_packet = vec![0; 1024];
        let _ = client.read(&mut init_packet).await.unwrap();
        client.shutdown().await.unwrap();
        let packet_size = u16::from_le_bytes(init_packet[..2].try_into().unwrap());
        let packet_id = init_packet[2];
        let revision = i32::from_le_bytes(init_packet[7..11].try_into().unwrap());
        assert_eq!(packet_size, 172);
        assert_eq!(packet_id, 0);
        assert_eq!(revision, 0x0000_c621);
    }
}

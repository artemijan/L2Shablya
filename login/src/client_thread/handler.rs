use crate::controller::LoginController;
use crate::packet::cp_factory::build_client_packet;
use crate::packet::to_client::Init;
use anyhow::bail;
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
use std::fmt;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, Notify};
use tracing::{info, instrument};

#[derive(Clone)]
pub struct Client {
    blowfish_key: Vec<u8>,
    pub ip: Ipv4Addr,
    tcp_reader: Arc<Mutex<dyn AsyncRead + Send + Unpin>>,
    tcp_writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    pub account_name: Option<String>,
    db_pool: DBPool,
    session_id: i32,
    shutdown_notifier: Arc<Notify>,
    lc: Arc<LoginController>,
    encryption: Encryption,
    timeout: u8,
    session_key: SessionKey,
    rsa_key_pair: rsa::ScrambledRSAKeyPair,
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("ip", &self.ip)
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

/// # This function called each time when there is a new connection.
/// ## Arguments
/// - stream - `TcpStream` to send/receive `shared_packets`
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
    pub fn set_session_key(&mut self, key: SessionKey) {
        self.session_key = key;
    }
    pub fn get_session_key(&self) -> &SessionKey {
        &self.session_key
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
    type ControllerType = LoginController;
    fn get_handler_name() -> &'static str {
        "Login client handler"
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.lc
    }

    fn new<R, W>(
        reader: R,
        writer: W,
        ip: Ipv4Addr,
        db_pool: DBPool,
        lc: Arc<Self::ControllerType>,
    ) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let blowfish_key = generate_blowfish_key(None);
        let encryption = Encryption::new(&blowfish_key.clone());
        let session_id = LoginController::generate_session_id();
        let timeout = lc.get_config().client.timeout;
        Self {
            tcp_reader: Arc::new(Mutex::new(reader)),
            tcp_writer: Arc::new(Mutex::new(writer)),
            db_pool,
            ip,
            shutdown_notifier: Arc::new(Notify::new()),
            encryption,
            session_id,
            session_key: SessionKey::new(),
            account_name: None,
            rsa_key_pair: lc.get_random_rsa_key_pair().clone(),
            blowfish_key,
            timeout,
            lc,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> anyhow::Result<()> {
        info!("Client connected: {:?}", self.ip);
        let mut init = Init::new(
            self.session_id,
            self.rsa_key_pair.get_scrambled_modulus(),
            self.blowfish_key.clone(),
        )?;
        self.tcp_writer
            .lock()
            .await
            .write_all(init.buffer.get_data_mut(false))
            .await
            .map_err(|_| Packet::UnableToSendInit)?;
        Ok(())
    }

    async fn on_disconnect(&mut self) {
        info!("Player disconnected: {:?}", self.session_id);
    }

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<dyn AsyncRead + Send + Unpin>> {
        &self.tcp_reader
    }

    fn get_timeout(&self) -> Option<u64> {
        Some(u64::from(self.timeout))
    }

    fn get_db_pool(&self) -> &DBPool {
        &self.db_pool
    }

    #[instrument(skip(self, data))]
    async fn on_receive_bytes(&mut self, _: usize, data: &mut [u8]) -> anyhow::Result<()> {
        self.encryption.decrypt(data)?;
        let handler = build_client_packet(data, &self.rsa_key_pair)?;
        handler.handle(self).await
    }
}

#[async_trait]
impl PacketSender for Client {
    async fn encrypt(&self, _: &mut [u8]) -> anyhow::Result<()> {
        Ok(())
    }
    fn is_encryption_enabled(&self) -> bool {
        false
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>> {
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
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    #[timeout(4000)]
    async fn test_init_packet_sent() {
        // Create a listener on a local port
        let db_pool = get_test_db().await;
        let (mut client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../test_data/test_config.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        // Spawn a server task to handle a single connection
        let handle = tokio::spawn(async move {
            let ip = Ipv4Addr::new(127, 0, 0, 1);
            let (r, w) = split(server);
            let mut ch = Client::new(r, w, ip, db_pool, cloned_lc);
            let res = ch.handle_client().await;
            (ch, res)
        });
        // Create a client to connect to the server
        // Read the response from the server
        let mut init_packet = vec![0; 1024];
        let _ = client.read(&mut init_packet).await.unwrap();
        let bytes = [
            22, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        client.write_all(&bytes).await.unwrap();
        client.shutdown().await.unwrap();
        let (h, res) = handle.await.unwrap();
        assert!(
            matches!(res, Err(e) if e.to_string() == "Error receiving packet: Unable to decrypt client packet")
        );
        assert!(!h.is_encryption_enabled());
        assert!(h
            .check_session(h.get_session_key().login_ok1, h.get_session_key().login_ok2)
            .is_ok());
        assert!(h
            .check_session(h.get_session_key().login_ok2, h.get_session_key().login_ok1)
            .is_err());
        let packet_size = u16::from_le_bytes(init_packet[..2].try_into().unwrap());
        let packet_id = init_packet[2];
        let revision = i32::from_le_bytes(init_packet[7..11].try_into().unwrap());
        assert_eq!(packet_size, 172);
        assert_eq!(packet_id, 0);
        assert_eq!(revision, 0x0000_c621);
    }
}

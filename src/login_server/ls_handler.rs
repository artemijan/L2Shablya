use crate::common::errors::PacketErrors;
use crate::common::errors::PacketErrors::{ClientPacketNotFound, UnableToSendInitPacket};
use crate::common::session::SessionKey;
use crate::crypt::blowfish_engine::generate_blowfish_key;
use crate::crypt::login::LoginEncryption;
use crate::crypt::rsa::ScrambledRSAKeyPair;
use crate::login_server::controller::{LoginController};
use crate::login_server::PacketHandler;
use crate::packet::to_client::Init;
use crate::packet::ls_factory::build_client_packet;
use anyhow::{Context, Error};
use async_trait::async_trait;
use rand::Rng;
use sqlx::AnyPool;
use std::sync::Arc;

use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct ClientHandler {
    blowfish_key: Vec<u8>,
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    pub account_name: Option<String>,
    db_pool: AnyPool,
    session_id: i32,
    pub lc: Arc<LoginController>,
    encryption: LoginEncryption,
    timeout: usize,
    session_key: SessionKey,
    connection_start_time: SystemTime,
    rsa_key_pair: ScrambledRSAKeyPair,
}

/// # This function called each time when there is a new connection.
/// ## Arguments
/// - stream - TcpStream to send/receive packets
/// - lc - LoginClient object is needed to store some client related data
///
/// # The whole login process looks like the following:
/// 1. send init ()
/// 2. wait request AuthGG
/// 3. AuthGG::new(session_id); // sen AuthGG OK
/// 4. wait request auth login
/// 5. LoginOk::new(session_key); //send Auth ok
/// 6. wait for RequestServerList
/// 7. Respond with ServerList::new()
///
impl ClientHandler {
    pub fn new(stream: TcpStream, db_pool: AnyPool, lc: Arc<LoginController>, timeout: usize) -> Self {
        let mut rng = rand::thread_rng();
        let blowfish_key = generate_blowfish_key();
        let encryption = LoginEncryption::new(&blowfish_key.clone());
        let session_id = rng.gen();
        let connection_start_time = SystemTime::now();
        let (tcp_reader, tcp_writer) = stream.into_split();
        ClientHandler {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            db_pool,
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
    pub fn get_session_id(&self) -> i32 {
        self.session_id
    }
    pub fn get_db_pool_mut(&mut self) -> &mut AnyPool {
        &mut self.db_pool
    }
    pub fn get_session_key(&self) -> &SessionKey {
        &self.session_key
    }
}

#[async_trait]
impl PacketHandler for ClientHandler {
    fn get_handler_name() -> String {
        "Login client handler".to_string()
    }

    fn get_lc(&self) -> &Arc<LoginController> {
        &self.lc
    }

    async fn on_connect(&mut self) -> Result<(), PacketErrors> {
        let addr = self.tcp_reader.lock().await.peer_addr().unwrap();
        println!("Client connected: {:?}", addr);
        let init = Init::new(
            self.session_id,
            self.rsa_key_pair.get_scrambled_modulus(),
            self.blowfish_key.clone(),
        );
        self.tcp_writer
            .lock()
            .await
            .write_all(&init.buffer.get_data())
            .await
            .map_err(|_| UnableToSendInitPacket)?;
        Ok(())
    }
    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        Some(self.timeout as u64)
    }

    async fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), Error> {
        self.tcp_writer
            .lock()
            .await
            .write_all(bytes.as_slice())
            .await
            .with_context(|| "Failed to flush packet to socket")
    }

    async fn on_receive_bytes(&mut self, packet_size: usize, data: &mut [u8]) -> Result<(), Error> {
        self.encryption.crypt.decrypt(data, 0, packet_size)?;
        let handler =
            build_client_packet(data, &self.rsa_key_pair).ok_or_else(|| ClientPacketNotFound {
                opcode: data[0] as usize,
            })?;
        let resp = handler.handle(self).await;
        self.handle_result(resp).await
    }
}

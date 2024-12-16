use l2_core::dto::InboundConnection;
use l2_core::errors::Packet;
use l2_core::packets::common::{GSLoginFail, PacketType};
use l2_core::packets::error::PacketRun;
use l2_core::packets::ls_2_gs::InitLS;
use l2_core::traits::handlers::{InboundHandler, PacketHandler};
use l2_core::traits::Shutdown;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use entities::DBPool;
use crate::controller::Login;
use l2_core::config::login::LoginServer;
use crate::gs_thread::enums;
use crate::message::Request;
use crate::packet::gs_factory::build_gs_packet;
use anyhow::{bail, Error};
use async_trait::async_trait;
use openssl::error::ErrorStack;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, Notify, RwLock};
use tracing::{info, instrument};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct GameServer {
    ///
    /// `tcp_reader` and `tcp_writer` are wrapped into Arc<Mutex> because we need to share the handler
    /// across two tokio threads:
    /// 1. Main thread to accept and answer packets
    /// 2. Listen for messages from Client login thread if we need info about logging in Player
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    shutdown_listener: Arc<Notify>,
    lc: Arc<Login>,
    db_pool: DBPool,
    key_pair: ScrambledRSAKeyPair,
    blowfish: Encryption,
    connection_state: enums::GS,
    pub server_id: Option<u8>,
    income_messages: Arc<RwLock<HashMap<String, Request>>>,
}

impl GameServer {
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) {
        self.blowfish = Encryption::from_u8_key(new_bf_key);
    }
    pub fn start_channel(&self) {
        let (rx, mut tx) = mpsc::channel::<(u8, Request)>(100);
        self.lc.connect_gs(self.server_id.unwrap(), rx);
        let inbox = self.income_messages.clone();
        let cloned_self = self.clone();
        let threshold = Duration::from_secs(u64::from(
            self.lc.get_config().listeners.game_servers.messages.timeout,
        ));
        tokio::spawn(async move {
            loop {
                if let Some((_, mut request)) = tx.recv().await {
                    let mut income_messages = inbox.write().await;
                    //the message has been sent already, there is no sense to do it twice
                    let existing_msg = income_messages.remove(&request.id);
                    if let Some(existing_msg) = existing_msg {
                        if let Some(resp) = existing_msg.response {
                            let _ = resp.send(None); // ignore error, we don't care if pipe is broken
                        }
                    }
                    //do a cleanup, if we have old messages, remove them
                    let now = SystemTime::now();

                    income_messages.retain(|_, req| {
                        now.duration_since(req.sent_at)
                            .map_or(false, |elapsed| elapsed <= threshold)
                    });
                    // send packet later, now we only remember it
                    let Some(req_body) = request.body.take() else {
                        continue;
                    };
                    // we are safe to send bytes firs and then update messages, there is a lock.
                    if cloned_self.send_packet(req_body).await.is_ok() {
                        income_messages.insert(request.id.clone(), request);
                    } else if let Some(resp) = request.response {
                        //if it wasn't successful then just send back NoResponse without storing it
                        let _ = resp.send(None);
                    }
                }
            }
        });
    }
    pub async fn respond_to_message(&self, message_id: &str, message: PacketType) {
        let mut msg_box = self.income_messages.write().await;
        let msg = msg_box.remove(message_id);
        // TODO: once combining if let will be stable, refactor it
        if let Some(request) = msg {
            if let Some(server_id) = self.server_id {
                if let Some(resp) = request.response {
                    resp.send(Some((server_id, message)))
                        .expect("Unable to send response");
                }
            }
        }
        //if message is missing then we just ignore it
    }
    pub async fn set_connection_state(&mut self, state: &enums::GS) -> Result<(), PacketRun> {
        if let Err(err) = self.connection_state.transition_to(state) {
            let err_msg = format!("Connection state transition failed {err:?}");
            self.send_packet(Box::new(GSLoginFail::new(err))).await?;
            return Err(PacketRun { msg: Some(err_msg) });
        }
        Ok(())
    }

    pub fn decrypt_rsa(&self, data: &mut [u8]) -> Result<Vec<u8>, ErrorStack> {
        self.key_pair.decrypt_data(data)
    }
}

impl Shutdown for GameServer {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_listener.clone()
    }
}

#[async_trait]
impl PacketHandler for GameServer {
    type ConfigType = LoginServer;
    type ControllerType = Login;
    fn get_handler_name() -> &'static str {
        "Game server handler"
    }
    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.lc
    }

    fn new(stream: TcpStream, db_pool: DBPool, lc: Arc<Self::ControllerType>) -> Self {
        let (tcp_reader, tcp_writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(tcp_writer));
        let reader = Arc::new(Mutex::new(tcp_reader));
        let cfg = lc.get_config();
        Self {
            tcp_reader: reader,
            tcp_writer: writer,
            db_pool,
            shutdown_listener: Arc::new(Notify::new()),
            key_pair: lc.get_random_rsa_key_pair(),
            blowfish: Encryption::from_u8_key(cfg.blowfish_key.as_bytes()),
            connection_state: enums::GS::Initial,
            lc,
            server_id: None,
            income_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> Result<(), Packet> {
        info!(
            "Game server connected: {:?}",
            self.tcp_reader.lock().await.peer_addr().unwrap()
        );
        self.connection_state = enums::GS::Connected;
        let init_packet = Box::new(InitLS::new(self.key_pair.get_modulus()));
        self.send_packet(init_packet).await?;
        Ok(())
    }

    fn on_disconnect(&mut self) {
        info!(
            "Game server disconnected: ID ({:})",
            self.server_id.unwrap_or_default()
        );
        if let Some(server_id) = self.server_id {
            let lc = self.get_controller();
            lc.remove_gs(server_id);
            lc.remove_all_gs_players(server_id);
        }
    }

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        None
    }
    fn get_db_pool_mut(&mut self) -> &mut DBPool {
        &mut self.db_pool
    }
    #[instrument(skip(self,bytes))]
    async fn on_receive_bytes(&mut self, _: usize, bytes: &mut [u8]) -> Result<(), Error> {
        self.blowfish.decrypt(bytes)?;
        if !Encryption::verify_checksum(bytes) {
            bail!("Can not verify check sum.")
        }
        let handler = build_gs_packet(bytes).ok_or_else(|| Packet::ClientPacketNotFound {
            opcode: bytes[0] as usize,
        })?;
        if let Err(error) = handler.handle(self).await {
            info!("Error handling packet: {error:?}");
            return Err(error.into());
        };
        Ok(())
    }
    fn encryption(&self) -> Option<&Encryption> {
        Some(&self.blowfish)
    }
}

impl InboundHandler for GameServer {
    type ConfigType = LoginServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
        &cfg.listeners.game_servers.connection
    }
}

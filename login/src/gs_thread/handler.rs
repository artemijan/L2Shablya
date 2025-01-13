use crate::controller::Login;
use crate::gs_thread::enums;
use crate::packet::gs_factory::build_gs_packet;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::DBPool;
use l2_core::config::login::LoginServer;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::dto::InboundConnection;
use l2_core::errors::Packet;
use l2_core::shared_packets::common::GSLoginFail;
use l2_core::shared_packets::error::PacketRun;
use l2_core::shared_packets::ls_2_gs::InitLS;
use l2_core::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use tracing::{info, instrument};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct GameServer {
    ///
    /// `tcp_reader` and `tcp_writer` are wrapped into Arc<Mutex> because we need to share the handler
    /// across two tokio threads:
    /// 1. Main thread to accept and answer `shared_packets`
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
}

impl GameServer {
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) {
        self.blowfish = Encryption::from_u8_key(new_bf_key);
    }

    pub async fn set_connection_state(&mut self, state: &enums::GS) -> Result<(), PacketRun> {
        if let Err(err) = self.connection_state.transition_to(state) {
            let err_msg = format!("Connection state transition failed {err:?}");
            self.send_packet(Box::new(GSLoginFail::new(err))).await?;
            return Err(PacketRun { msg: Some(err_msg) });
        }
        Ok(())
    }

    pub fn decrypt_rsa(&self, data: &mut [u8]) -> anyhow::Result<Vec<u8>> {
        self.key_pair.decrypt_data(data)
    }
}

impl Shutdown for GameServer {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_listener.clone()
    }
}

#[async_trait]
impl PacketSender for GameServer {
    fn encryption(&self) -> Option<&Encryption> {
        Some(&self.blowfish)
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
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
            lc.message_broker.unregister_packet_handler(server_id);
            lc.remove_all_gs_players(server_id);
        }
    }

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }

    fn get_timeout(&self) -> Option<u64> {
        None
    }
    fn get_db_pool(&self) -> &DBPool {
        &self.db_pool
    }
    #[instrument(skip(self, bytes))]
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
}

impl InboundHandler for GameServer {
    type ConfigType = LoginServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
        &cfg.listeners.game_servers.connection
    }
}

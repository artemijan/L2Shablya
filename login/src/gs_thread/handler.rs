use crate::controller::LoginController;
use crate::gs_thread::enums;
use crate::packet::gs_factory::build_gs_packet;
use anyhow::{anyhow, bail, Error};
use async_trait::async_trait;
use entities::DBPool;
use l2_core::config::login::LoginServer;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::dto::InboundConnection;
use l2_core::shared_packets::common::GSLoginFail;
use l2_core::shared_packets::ls_2_gs::InitLS;
use l2_core::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use macro_common::LoginPacketSenderImpl;
use std::fmt;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{Mutex, Notify};
use tracing::{info, instrument};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, LoginPacketSenderImpl)]
pub struct GameServer {
    ///
    /// `tcp_reader` and `tcp_writer` are wrapped into Arc<Mutex> because we need to share the handler
    /// across two tokio threads:
    /// 1. Main thread to accept and answer `shared_packets`
    /// 2. Listen for messages from Client login thread if we need info about logging in Player
    tcp_reader: Arc<Mutex<dyn AsyncRead + Send + Unpin>>,
    tcp_writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    shutdown_listener: Arc<Notify>,
    lc: Arc<LoginController>,
    db_pool: DBPool,
    ip: Ipv4Addr,
    key_pair: ScrambledRSAKeyPair,
    blowfish: Encryption,
    connection_state: enums::GS,
    pub server_id: Option<u8>,
}
impl fmt::Debug for GameServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("ip", &self.ip)
            .field("connection_state", &self.connection_state)
            .field("server_id", &self.server_id)
            .finish_non_exhaustive()
    }
}
impl GameServer {
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) ->anyhow::Result<()> {
        self.blowfish = Encryption::try_from_u8_key(new_bf_key)?;
        Ok(())
    }
    pub fn set_rsa_key(&mut self, new_key: ScrambledRSAKeyPair) {
        self.key_pair = new_key;
    }
    pub fn try_get_server_id(&self) -> anyhow::Result<u8> {
        self.server_id
            .ok_or_else(|| anyhow!("Possible cheating: No server ID set"))
    }
    pub async fn set_connection_state(&mut self, state: &enums::GS) -> anyhow::Result<()> {
        if let Err(err) = self.connection_state.transition_to(state) {
            let err_msg = format!("Connection state transition failed {err:?}");
            self.send_packet(Box::new(GSLoginFail::new(err)?)).await?;
            bail!(err_msg);
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
impl PacketHandler for GameServer {
    type ConfigType = LoginServer;
    type ControllerType = LoginController;
    fn get_handler_name() -> &'static str {
        "Game server handler"
    }
    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.lc
    }

    fn new<R, W>(r: R, w: W, ip: Ipv4Addr, db_pool: DBPool, lc: Arc<Self::ControllerType>) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let cfg = lc.get_config();
        Self {
            tcp_reader: Arc::new(Mutex::new(r)),
            tcp_writer: Arc::new(Mutex::new(w)),
            db_pool,
            ip,
            shutdown_listener: Arc::new(Notify::new()),
            key_pair: lc.get_random_rsa_key_pair().clone(), // we have to clone it as we need ownership
            blowfish: Encryption::from_u8_key(cfg.blowfish_key.as_bytes()),
            connection_state: enums::GS::Initial,
            lc,
            server_id: None,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> anyhow::Result<()> {
        info!("Game server connected: {:?}", self.ip);
        self.connection_state = enums::GS::Connected;
        let init_packet = Box::new(InitLS::new(self.key_pair.get_modulus()));
        self.send_packet(init_packet).await?;
        Ok(())
    }

    async fn on_disconnect(&mut self) {
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

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<dyn AsyncRead + Send + Unpin>> {
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
        let handler = build_gs_packet(bytes)?;
        handler.handle(self).await
    }
}

impl InboundHandler for GameServer {
    type ConfigType = LoginServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
        &cfg.listeners.game_servers.connection
    }
}

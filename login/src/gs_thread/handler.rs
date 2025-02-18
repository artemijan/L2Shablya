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

#[cfg(not(tarpaulin_include))]
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
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) -> anyhow::Result<()> {
        self.blowfish = Encryption::try_from_u8_key(new_bf_key)?;
        Ok(())
    }

    #[cfg(test)]
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
            self.server_id = None;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gs_thread::GSHandler;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    #[timeout(3000)]
    async fn test_gs_connect_disconnect() {
        let db_pool = get_test_db().await;
        let (mut client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut gs_handler = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        gs_handler.server_id = Some(1);
        let p_key =
            ScrambledRSAKeyPair::from_pem(include_str!("../../../test_data/test_private_key.pem"))
                .unwrap();
        let pub_key = p_key.to_public_key();
        let scr = ScrambledRSAKeyPair::new((p_key, pub_key));
        gs_handler.set_rsa_key(scr);
        gs_handler.on_connect().await.unwrap();
        let mut p = [0u8; 146];
        client.read_exact(&mut p).await.unwrap();
        assert_eq!(
            [
                146, 0, 12, 137, 38, 73, 202, 146, 136, 195, 228, 30, 213, 177, 109, 96, 57, 167,
                207, 66, 183, 249, 25, 6, 120, 55, 123, 122, 93, 236, 249, 157, 198, 122, 165, 225,
                23, 224, 111, 135, 241, 205, 123, 32, 159, 3, 12, 161, 151, 61, 146, 194, 42, 16,
                238, 52, 170, 16, 107, 247, 161, 5, 89, 210, 153, 113, 93, 180, 103, 99, 210, 49,
                219, 193, 51, 90, 35, 187, 167, 26, 167, 113, 82, 233, 122, 171, 110, 209, 254, 89,
                168, 46, 49, 78, 25, 224, 148, 171, 129, 105, 241, 250, 202, 111, 3, 13, 81, 142,
                108, 190, 58, 118, 129, 116, 119, 248, 86, 218, 3, 210, 205, 1, 133, 58, 109, 190,
                219, 123, 8, 118, 184, 29, 85, 0, 49, 221, 79, 67, 200, 255, 35, 230, 46, 231, 23,
                148
            ],
            p
        );
        let mut blow_fish = [
            100, 231, 252, 211, 54, 62, 224, 27, 176, 228, 163, 126, 28, 227, 64, 163, 102, 81,
            159, 223, 128, 192, 98, 33, 75, 160, 99, 225, 219, 57, 131, 145, 85, 214, 187, 26, 250,
            151, 35, 230, 86, 31, 70, 166, 71, 15, 173, 216, 240, 58, 5, 179, 149, 166, 149, 220,
            208, 15, 5, 99, 228, 45, 126, 114, 68, 16, 221, 153, 38, 249, 183, 46, 135, 110, 146,
            167, 110, 206, 202, 63, 127, 167, 44, 144, 218, 93, 10, 225, 198, 8, 100, 212, 180,
            247, 177, 36, 144, 69, 163, 231, 222, 247, 160, 87, 71, 150, 77, 188, 16, 43, 57, 60,
            52, 27, 131, 235, 91, 185, 94, 24, 247, 42, 14, 131, 163, 54, 182, 1, 74, 79, 211, 249,
            5, 30, 124, 176, 44, 123, 102, 79, 64, 22, 166, 132,
        ];
        let res = gs_handler.on_receive_bytes(1, &mut blow_fish).await;
        assert!(res.is_ok());
        blow_fish[0] = 1;
        let res = gs_handler.on_receive_bytes(1, &mut blow_fish).await;
        assert!(res.is_err());
        gs_handler.on_disconnect().await;
        assert!(gs_handler.server_id.is_none());
        client.shutdown().await.unwrap();
    }
}

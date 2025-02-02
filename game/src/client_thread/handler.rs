use crate::controller::Controller;
use crate::cp_factory::build_client_packet;
use crate::ls_thread::LoginHandler;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::dao::char_info::CharacterInfo;
use entities::entities::{character, user};
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::dto::InboundConnection;
use l2_core::session::SessionKey;
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use l2_core::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use std::fmt;
use std::future::Future;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{Mutex, Notify};
use tracing::{error, info, instrument};

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub enum ClientStatus {
    Connected,
    Closing,
    Entering,
    Authenticated,
    Disconnected,
    InGame,
}
#[allow(clippy::module_name_repetitions)]
pub struct ClientHandler {
    tcp_reader: Arc<Mutex<dyn AsyncRead + Unpin + Send>>,
    tcp_writer: Arc<Mutex<dyn AsyncWrite + Unpin + Send>>,
    db_pool: DBPool,
    controller: Arc<Controller>,
    shutdown_notifier: Arc<Notify>,
    timeout: u8,
    ip: Ipv4Addr,
    blowfish: Option<Encryption>,
    protocol: Option<i32>,
    status: ClientStatus,
    account_chars: Option<Vec<CharacterInfo>>,
    selected_char: Option<i32>,
    session_key: Option<SessionKey>,
    user: Option<user::Model>,
}
impl fmt::Debug for ClientHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("ip", &self.ip)
            .field("protocol", &self.protocol)
            .field("status", &self.status)
            .field("selected_char", &self.selected_char)
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}

impl ClientHandler {
    pub fn get_protocol(&self) -> Option<i32> {
        self.protocol
    }
    pub fn set_protocol(&mut self, protocol: i32) -> anyhow::Result<()> {
        let cfg = self.controller.get_cfg();
        if self.status != ClientStatus::Connected {
            bail!("Invalid client status");
        }
        if !cfg.allowed_revisions.contains(&protocol) {
            bail!("Invalid protocol version {}", protocol);
        }
        self.protocol = Some(protocol);
        Ok(())
    }
    pub fn set_session_key(&mut self, session_key: SessionKey) {
        self.session_key = Some(session_key);
    }

    pub fn try_get_user(&self) -> anyhow::Result<&user::Model> {
        self.user.as_ref().ok_or(anyhow::anyhow!("User not set"))
    }
    pub fn get_user(&self) -> Option<&user::Model> {
        self.user.as_ref()
    }
    pub fn set_user(&mut self, user: user::Model) {
        self.user = Some(user);
    }

    pub fn get_session_key(&self) -> Option<&SessionKey> {
        self.session_key.as_ref()
    }
    pub fn try_get_session_key(&self) -> anyhow::Result<&SessionKey> {
        self.session_key
            .as_ref()
            .ok_or(anyhow::anyhow!("Programming error - Session is missing"))
    }
    pub fn set_status(&mut self, status: ClientStatus) {
        self.status = status;
    }

    pub fn set_account_chars(&mut self, chars: Vec<CharacterInfo>) {
        self.account_chars = Some(chars);
    }

    pub fn get_account_chars(&self) -> Option<&Vec<CharacterInfo>> {
        self.account_chars.as_ref()
    }
    pub fn try_get_account_chars(&self) -> anyhow::Result<&Vec<CharacterInfo>> {
        self.account_chars.as_ref().ok_or(anyhow::anyhow!(
            "Programming error, missing account characters"
        ))
    }

    pub fn select_char(&mut self, char_slot: i32) {
        self.selected_char = Some(char_slot);
    }

    pub fn get_selected_char_slot(&self) -> Option<i32> {
        self.selected_char
    }

    pub fn add_character(&mut self, character: CharacterInfo) -> anyhow::Result<()> {
        self.account_chars
            .as_mut()
            .ok_or(anyhow::anyhow!(
                "Programming error, or possible cheating - missing characters."
            ))?
            .push(character);
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub async fn with_char_by_slot_id<F, Fut>(
        &mut self,
        slot_id: i32,
        modify_fn: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(character::Model) -> Fut,
        Fut: Future<Output = anyhow::Result<character::Model>> + Send,
    {
        if let Some(chars) = self.account_chars.as_mut() {
            if slot_id >= i32::try_from(chars.len())? || slot_id < 0 {
                bail!("Missing character at slot: {slot_id}")
            }
            let mut char_info: CharacterInfo = chars.remove(slot_id as usize);
            let model = char_info.char_model.clone();
            let updated_char = modify_fn(model).await?;
            char_info.char_model = updated_char;
            chars.insert(slot_id as usize, char_info);
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn try_get_char_by_slot_id(&self, slot_id: i32) -> anyhow::Result<&CharacterInfo> {
        self.account_chars
            .as_ref()
            .ok_or(anyhow::anyhow!(
                "Possible programming error or cheating: Characters not set"
            ))?
            .get(slot_id as usize)
            .ok_or(anyhow::anyhow!("Missing character at slot {slot_id}"))
    }

    pub fn set_encryption(&mut self, bf_key: Option<Encryption>) {
        self.blowfish = bf_key;
    }
    pub fn generate_key() -> Vec<u8> {
        let mut key = generate_blowfish_key(None);
        key[8] = 0xc8;
        key[9] = 0x27;
        key[10] = 0x93;
        key[11] = 0x01;
        key[12] = 0xa1;
        key[13] = 0x6c;
        key[14] = 0x31;
        key[15] = 0x97;
        key
    }
}
impl Shutdown for ClientHandler {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_notifier.clone()
    }
}

#[async_trait]
impl PacketSender for ClientHandler {
    fn encryption(&self) -> Option<&Encryption> {
        self.blowfish.as_ref()
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Unpin + Send>> {
        &self.tcp_writer
    }
}

#[async_trait]
impl PacketHandler for ClientHandler {
    type ConfigType = GSServer;
    type ControllerType = Controller;

    fn get_handler_name() -> &'static str {
        "Client handler"
    }

    fn get_controller(&self) -> &Arc<Self::ControllerType> {
        &self.controller
    }

    fn new<R, W>(
        r: R,
        w: W,
        ip: Ipv4Addr,
        db_pool: DBPool,
        controller: Arc<Self::ControllerType>,
    ) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let cfg = controller.get_cfg();
        Self {
            tcp_reader: Arc::new(Mutex::new(r)),
            tcp_writer: Arc::new(Mutex::new(w)),
            shutdown_notifier: Arc::new(Notify::new()),
            timeout: cfg.client.timeout,
            status: ClientStatus::Connected,
            controller,
            db_pool,
            ip,
            blowfish: None,
            account_chars: None,
            protocol: None,
            user: None,
            session_key: None,
            selected_char: None,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> anyhow::Result<()> {
        info!("Client connected.");
        Ok(())
    }

    async fn on_disconnect(&mut self) {
        info!("Disconnecting Client...");
        let Some(user) = self.user.as_ref() else {
            return;
        };
        self.controller.logout_account(&user.username);
        let packet = match PlayerLogout::new(&user.username) {
            Err(e) => {
                error!("Cannot build logout packet: {}", e);
                //exit function
                return;
            }
            Ok(p) => p,
        };

        if let Err(err) = self
            .controller
            .message_broker
            .notify(LoginHandler::HANDLER_ID, Box::new(packet))
            .await
        {
            error!(
                "Error while sending logout to login server, cause: {:?}",
                err
            );
        }
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

    #[instrument(skip(self, bytes))]
    async fn on_receive_bytes(&mut self, _: usize, bytes: &mut [u8]) -> Result<(), Error> {
        if let Some(blowfish) = self.encryption() {
            blowfish.decrypt(bytes)?;
            if !Encryption::verify_checksum(bytes) {
                bail!("Can not verify check sum.");
            }
        }

        let handler = build_client_packet(bytes)?;
        handler.handle(self).await
    }
}

impl InboundHandler for ClientHandler {
    type ConfigType = GSServer;

    fn get_connection_config(cfg: &Self::ConfigType) -> &InboundConnection {
        &cfg.listeners.clients.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::from_client::protocol::ProtocolVersion;
    use l2_core::shared_packets::common::{ReadablePacket, SendablePacket};
    use l2_core::shared_packets::write::SendablePacketBuffer;
    use l2_core::tests::get_test_db;
    use l2_core::traits::ServerConfig;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt, DuplexStream};
    use tokio::task::JoinHandle;

    impl ProtocolVersion {
        pub fn new(version: i32) -> anyhow::Result<Self> {
            let mut inst = Self {
                version,
                buffer: SendablePacketBuffer::new(),
            };
            inst.buffer.write(Self::PACKET_ID)?;
            inst.buffer.write_i32(inst.version)?;
            Ok(inst)
        }
    }

    fn build_client_handler(server: DuplexStream) -> JoinHandle<anyhow::Result<()>> {
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        let cloned_controller = controller.clone();
        // Spawn a server task to handle a single connection
        tokio::spawn(async move {
            let db_pool = get_test_db().await;
            let ip = Ipv4Addr::new(127, 0, 0, 1);
            let (r, w) = split(server);
            let mut ch = ClientHandler::new(r, w, ip, db_pool, cloned_controller);
            ch.handle_client().await
        })
    }

    #[tokio::test]
    async fn test_protocol_version_fail() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(1024);
        let mut login_packet = ProtocolVersion::new(6_553_697).unwrap();
        let bytes = login_packet.get_bytes(false);
        bytes[3] = 1;
        bytes[4] = 2;
        let _ = client.write(bytes).await.unwrap();
        let h = build_client_handler(server);
        client.shutdown().await.unwrap();
        let err = h.await.unwrap();
        assert!(err.is_err());
        let err_str = err.err().unwrap().to_string();
        println!("{err_str}");
        assert!(err_str.contains("Invalid protocol version"));
    }

    #[tokio::test]
    async fn test_protocol_version_success() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(2024);
        // Read the response from the server
        let mut login_packet = ProtocolVersion::new(6_553_697).unwrap();
        let bytes = login_packet.get_bytes(false);
        let _ = client.write(bytes).await.unwrap();
        client.flush().await.unwrap();
        let h = build_client_handler(server);
        let mut resp = [0; 4];
        client.read_exact(&mut resp).await.unwrap();
        println!("{resp:?}");
        client.shutdown().await.unwrap();
        let _ = h.await.unwrap();
        assert_eq!(
            resp,
            [26, 0, 46, 1]
        );
    }
}

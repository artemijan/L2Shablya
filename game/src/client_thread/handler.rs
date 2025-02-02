use crate::controller::Controller;
use crate::cp_factory::build_client_packet;
use crate::ls_thread::LoginHandler;
use anyhow::{bail, Error};
use async_trait::async_trait;
use entities::dao::char_info::CharacterInfo;
use entities::entities::{character, user};
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::crypt::game::GameClientEncryption;
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
    blowfish: Option<Arc<Mutex<GameClientEncryption>>>,
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

    pub fn set_encryption(&mut self, bf_key: Option<GameClientEncryption>) {
        if let Some(key) = bf_key {
            self.blowfish = Some(Arc::new(Mutex::new(key)));
        } else {
            self.blowfish = None;
        }
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
    async fn encrypt(&self, bytes: &mut [u8]) -> anyhow::Result<()> {
        if let Some(bf) = self.blowfish.as_ref() {
            let size = bytes.len();
            Encryption::append_checksum(&mut bytes[2..size]);
            bf.lock().await.encrypt(&mut bytes[2..size]);
        }
        Ok(())
    }

    fn is_encryption_enabled(&self) -> bool {
        self.blowfish.is_some()
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
        if let Some(blowfish) = self.blowfish.as_ref() {
            blowfish.lock().await.decrypt(bytes)?;
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
    use crate::packets::from_client::auth::AuthLogin;
    use crate::packets::from_client::protocol::ProtocolVersion;
    use crate::packets::to_client::PlayerLoginResponse;
    use l2_core::shared_packets::common::{PacketType, ReadablePacket, SendablePacket};
    use l2_core::shared_packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
    use l2_core::shared_packets::ls_2_gs::PlayerAuthResponse;
    use l2_core::shared_packets::read::ReadablePacketBuffer;
    use l2_core::shared_packets::write::SendablePacketBuffer;
    use l2_core::tests::get_test_db;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use sea_orm::{ActiveModelTrait, ActiveValue, TryIntoModel};
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
    impl AuthLogin {
        pub fn new() -> anyhow::Result<Self> {
            let mut inst = Self {
                login_name: "test".to_string(),
                play_key_1: 0,
                play_key_2: 1,
                login_key_1: 2,
                login_key_2: 3,
                buffer: SendablePacketBuffer::new(),
            };
            inst.buffer.write(Self::PACKET_ID)?;
            inst.buffer.write_c_utf16le_string(Some(&inst.login_name))?;
            inst.buffer.write_i32(inst.play_key_1)?;
            inst.buffer.write_i32(inst.play_key_2)?;
            inst.buffer.write_i32(inst.login_key_1)?;
            inst.buffer.write_i32(inst.login_key_2)?;
            Ok(inst)
        }
    }

    fn build_client_handler(
        server: DuplexStream,
        controller_opt: Option<Arc<Controller>>,
    ) -> JoinHandle<(ClientHandler, anyhow::Result<()>)> {
        let controller = controller_opt.unwrap_or_else(|| {
            let cfg = Arc::new(GSServer::from_string(include_str!(
                "../../test_data/game.yaml"
            )));
            Arc::new(Controller::new(cfg))
        });
        let cloned_controller = controller.clone();
        // Spawn a server task to handle a single connection
        tokio::spawn(async move {
            let db_pool = get_test_db().await;
            let ip = Ipv4Addr::new(127, 0, 0, 1);
            let (r, w) = split(server);
            let mut ch = ClientHandler::new(r, w, ip, db_pool, cloned_controller);
            let r = ch.handle_client().await;
            (ch, r)
        })
    }

    #[tokio::test]
    #[timeout(2000)]
    async fn test_protocol_version_fail() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(1024);
        let h = build_client_handler(server, None);
        let mut login_packet = ProtocolVersion::new(6_553_697).unwrap();
        let bytes = login_packet.get_bytes(false);
        bytes[3] = 1;
        bytes[4] = 2;
        client.write_all(bytes).await.unwrap();
        client.shutdown().await.unwrap();
        let (_, err) = h.await.unwrap();
        assert!(err.is_err());
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("Invalid protocol version"));
    }

    #[tokio::test]
    #[timeout(2000)]
    async fn test_protocol_version_success() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(2024);
        let h = build_client_handler(server, None);
        let mut login_packet = ProtocolVersion::new(6_553_697).unwrap();
        let bytes = login_packet.get_bytes(false);
        client.write_all(bytes).await.unwrap();
        let mut resp = [0; 24];
        // Read the response from the server
        client.read_exact(&mut resp).await.unwrap();
        client.shutdown().await.unwrap();
        let (ch, _) = h.await.unwrap();
        assert_eq!(ch.protocol, Some(6_553_697));
        assert_eq!(resp[0..4], [26, 0, 46, 1]);
    }

    #[tokio::test]
    #[timeout(2000)]
    async fn test_auth_fail() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(2024);
        let h = build_client_handler(server, None);
        let mut auth = AuthLogin::new().unwrap();
        client.write_all(auth.get_bytes(false)).await.unwrap();
        client.shutdown().await.unwrap();
        let (_, err) = h.await.unwrap();
        assert!(err.is_err());
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("Protocol version not set"));
    }
    struct TestPacketSender {
        writer: Arc<Mutex<dyn AsyncWrite + Unpin + Send>>,
    }
    impl fmt::Debug for TestPacketSender {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestPacketSender")
        }
    }

    #[async_trait]
    impl PacketSender for TestPacketSender {
        async fn encrypt(&self, _: &mut [u8]) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_encryption_enabled(&self) -> bool {
            false
        }

        async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>> {
            &self.writer
        }
    }

    /// This test is testing whole logic for authenticating user from Protocol version
    /// and authenticating on login server.
    /// I decided to do integration test instead of small unit tests just to be able to change
    /// internals while still not braking functionality.
    #[tokio::test]
    #[timeout(2000)]
    async fn test_integration_auth_ok() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(1024);
        let (login_client, mut login_server) = tokio::io::duplex(1024);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../test_data/game.yaml"
        )));
        let user_record = user::ActiveModel {
            id: ActiveValue::NotSet,
            username: ActiveValue::Set("test".to_string()),
            password: ActiveValue::Set("password_hash".to_string()),
            access_level: ActiveValue::Set(0),
            ban_duration: ActiveValue::NotSet,
            ban_ip: ActiveValue::NotSet,
        };
        let user_model = user_record.save(&get_test_db().await).await.unwrap();
        let controller = Arc::new(Controller::new(cfg));
        let test_packet_sender = Arc::new(TestPacketSender {
            writer: Arc::new(Mutex::new(login_client)),
        });
        controller
            .message_broker
            .register_packet_handler(LoginHandler::HANDLER_ID, test_packet_sender.clone());
        let handle = build_client_handler(server, Some(controller.clone()));
        let mut login_packet = ProtocolVersion::new(6_553_697).unwrap();
        let bytes = login_packet.get_bytes(false);
        client.write_all(bytes).await.unwrap();
        let mut protocol_response = [0; 26];
        client.read_exact(&mut protocol_response).await.unwrap();
        assert_eq!(protocol_response[0], 26);
        let mut auth = AuthLogin::new().unwrap();
        client.write_all(auth.get_bytes(false)).await.unwrap();
        let mut auth_login_packet = [0; 29];
        login_server
            .read_exact(&mut auth_login_packet)
            .await
            .unwrap();
        let p = PlayerAuthRequest::read(&auth_login_packet[2..]).unwrap();
        assert_eq!(p.account_name, "test");
        assert_eq!(p.session.play_ok1, 1); //it should be vice versa
        assert_eq!(p.session.play_ok2, 0);
        assert_eq!(p.session.login_ok1, 2);
        assert_eq!(p.session.login_ok2, 3);
        let auth_ok_packet = PlayerAuthResponse::new("test", true);
        //login ok
        controller.message_broker.respond_to_message(
            Some(LoginHandler::HANDLER_ID),
            "test",
            PacketType::PlayerAuthResp(auth_ok_packet),
        );
        let mut player_in_game = [0; 15];
        login_server.read_exact(&mut player_in_game).await.unwrap();
        let pig = PlayerInGame::read(&player_in_game[2..]).unwrap();
        assert_eq!(pig.accounts, ["test"]);
        let mut auth_ok_resp = [0; 11];
        client.read_exact(&mut auth_ok_resp).await.unwrap();
        let mut buffer = ReadablePacketBuffer::new(&auth_ok_resp[2..]);
        let p_id = buffer.read_byte().unwrap();
        let is_ok = buffer.read_i32().unwrap();
        let reason = buffer.read_u32().unwrap();
        assert_eq!(PlayerLoginResponse::PACKET_ID, p_id);
        assert_eq!(is_ok, -1);
        assert_eq!(reason, 0);
        client.shutdown().await.unwrap();
        let (ch, _) = handle.await.unwrap();
        assert_eq!(ch.protocol, Some(6_553_697));
        assert_eq!(ch.session_key.unwrap(), p.session);
        assert_eq!(ch.user.unwrap(), user_model.try_into_model().unwrap());
    }
}

use std::future::Future;
use std::pin::Pin;
use crate::controller::Controller;
use crate::cp_factory::build_client_packet;
use crate::ls_thread::LoginHandler;
use anyhow::{anyhow, bail, Error};
use async_trait::async_trait;
use entities::dao::char_info::CharacterInfo;
use entities::entities::{character, user};
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::dto::InboundConnection;
use l2_core::errors::Packet;
use l2_core::session::SessionKey;
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use l2_core::traits::handlers::{InboundHandler, PacketHandler, PacketSender};
use l2_core::traits::Shutdown;
use std::sync::Arc;
use sea_orm::DbErr;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
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
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ClientHandler {
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    db_pool: DBPool,
    controller: Arc<Controller>,
    shutdown_notifier: Arc<Notify>,
    timeout: u8,
    blowfish: Option<Encryption>,
    protocol: Option<i32>,
    status: ClientStatus,
    account_chars: Option<Vec<CharacterInfo>>,
    session_key: Option<SessionKey>,
    pub user: Option<user::Model>,
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

    pub fn get_session_key(&self) -> Option<&SessionKey> {
        self.session_key.as_ref()
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

    pub fn try_get_account_chars_mut(&mut self) -> anyhow::Result<&mut Vec<CharacterInfo>> {
        self.account_chars.as_mut().ok_or(anyhow::anyhow!(
            "Programming error, or possible cheating - missing characters."
        ))
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
    pub async fn with_char_by_slot_id<F, Fut>(&mut self, slot_id: i32, modify_fn: F) -> anyhow::Result<()>
    where
        F: FnOnce(character::Model) -> Fut,
        Fut: Future<Output = anyhow::Result<character::Model, DbErr>> + Send + 'static
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

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
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

    fn new(stream: TcpStream, db_pool: DBPool, controller: Arc<Self::ControllerType>) -> Self {
        let (tcp_reader, tcp_writer) = stream.into_split();
        let cfg = controller.get_cfg();
        Self {
            tcp_reader: Arc::new(Mutex::new(tcp_reader)),
            tcp_writer: Arc::new(Mutex::new(tcp_writer)),
            shutdown_notifier: Arc::new(Notify::new()),
            controller,
            db_pool,
            blowfish: None,
            account_chars: None,
            timeout: cfg.client.timeout,
            protocol: None,
            status: ClientStatus::Connected,
            user: None,
            session_key: None,
        }
    }

    #[instrument(skip(self))]
    async fn on_connect(&mut self) -> Result<(), Packet> {
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

    fn get_stream_reader_mut(&self) -> &Arc<Mutex<OwnedReadHalf>> {
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

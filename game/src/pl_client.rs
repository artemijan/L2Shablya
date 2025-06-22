use crate::controller::Controller;
use crate::cp_factory::build_client_packet;
use anyhow::{anyhow, bail};
use entities::entities::{character, user};
use entities::DBPool;
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::{ActorStopReason, PanicError};
use kameo::message::{Context, Message};
use kameo::Actor;
use l2_core::crypt::game::GameClientEncryption;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::game_objects::player::Player;
use l2_core::network::connection::{send_packet, ConnectionActor, HandleIncomingPacket};
use l2_core::session::SessionKey;
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use std::fmt;
use std::future::Future;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Duration;
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, info};
use l2_core::shared_packets::write::SendablePacketBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum ClientStatus {
    Connected,
    Closing,
    Entering,
    Authenticated,
    Disconnected,
    InGame,
}
pub struct PlayerClient {
    pub db_pool: DBPool,
    pub controller: Arc<Controller>,
    pub ip: Ipv4Addr,
    blowfish: Option<GameClientEncryption>,
    protocol: Option<i32>,
    status: ClientStatus,
    account_chars: Option<Vec<Player>>,
    selected_char: Option<i32>,
    pub packet_sender: Option<ActorRef<ConnectionActor<Self>>>,
    session_key: Option<SessionKey>,
    user: Option<user::Model>,
}

impl fmt::Debug for PlayerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player client")
            .field("ip", &self.ip)
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}
impl PlayerClient {
    pub fn new(ip: Ipv4Addr, controller: Arc<Controller>, db_pool: DBPool) -> Self {
        Self {
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
            packet_sender: None,
        }
    }
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
    pub fn get_status(&self) -> &ClientStatus {
        &self.status
    }

    pub fn set_account_chars(&mut self, chars: Vec<Player>) {
        self.account_chars = Some(chars);
    }

    pub fn get_account_chars(&self) -> Option<&Vec<Player>> {
        self.account_chars.as_ref()
    }
    pub fn try_get_account_chars(&self) -> anyhow::Result<&Vec<Player>> {
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

    pub fn add_character(&mut self, character: Player) -> anyhow::Result<()> {
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
            let mut char_info: Player = chars.remove(slot_id as usize);
            let model = char_info.char_model.clone();
            let updated_char = modify_fn(model).await?;
            char_info.char_model = updated_char;
            chars.insert(slot_id as usize, char_info);
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn try_get_char_by_slot_id(&self, slot_id: i32) -> anyhow::Result<&Player> {
        self.account_chars
            .as_ref()
            .ok_or(anyhow::anyhow!(
                "Possible programming error or cheating: Characters not set"
            ))?
            .get(slot_id as usize)
            .ok_or(anyhow::anyhow!("Missing character at slot {slot_id}"))
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn try_get_selected_char(&self) -> anyhow::Result<&Player> {
        self.try_get_char_by_slot_id(
            self.selected_char
                .ok_or(anyhow!("Chars not set, possible cheating"))?,
        )
    }

    pub fn set_encryption(&mut self, bf_key: Option<GameClientEncryption>) {
        if let Some(key) = bf_key {
            self.blowfish = Some(key);
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
    fn encrypt(&mut self, bytes: &mut BytesMut) -> anyhow::Result<()> {
        if let Some(bf) = self.blowfish.as_mut() {
            let size = bytes.len();
            Encryption::append_checksum(&mut bytes[2..size]);
            bf.encrypt(&mut bytes[2..size])?;
        }
        Ok(())
    }
    fn is_encryption_enabled(&self) -> bool {
        self.blowfish.is_some()
    }
    pub async fn send_packet(&mut self, mut buffer: SendablePacketBuffer) -> anyhow::Result<()> {
        let mut data;
        if self.is_encryption_enabled() {
            buffer.write_padding()?;
            data = buffer.take();
            self.encrypt(&mut data)?;
        }else {
            data = buffer.take();
        }
        send_packet(self.packet_sender.as_ref(), data.freeze()).await
    }
}

impl Actor for PlayerClient {
    type Args = (
        Self,
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, ls_actor: ActorRef<Self>) -> anyhow::Result<Self> {
        let (mut state, reader, writer) = args;
        info!("Player client {} started: ", state.ip);
        let connection = ConnectionActor::spawn(ConnectionActor::new(
            ls_actor,
            state.ip,
            reader,
            writer,
            Duration::from_secs(0),
        ));
        connection.wait_for_startup().await;
        state.packet_sender = Some(connection);
        Ok(state)
    }
    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> anyhow::Result<ControlFlow<ActorStopReason>> {
        error!("Player client {} panicked: {:?}", self.ip, &err);
        Ok(ControlFlow::Break(ActorStopReason::Panicked(err)))
    }
    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> anyhow::Result<()> {
        info!("Disconnecting Client...");
        if let Some(s) = self.packet_sender.as_ref() {
            let _ = s.stop_gracefully().await; //ignore errors is it is already dead
            s.wait_for_shutdown().await;
        }
        let Some(user) = self.user.as_ref() else {
            return Ok(());
        };
        self.controller.logout_account(&user.username);
        let packet = match PlayerLogout::new(&user.username) {
            Err(e) => {
                error!("Cannot build logout packet: {}", e);
                //exit function
                return Ok(());
            }
            Ok(p) => p,
        };

        if let Some(ls_actor) = self.controller.get_ls_actor().await {
            if let Err(err) = ls_actor.tell(packet).await {
                error!(
                    "Error while sending logout to login server, cause: {:?}",
                    err
                );
            }
        } else {
            info!("Login server not found, so no need to send logout packet");
        }
        Ok(())
    }
    
}
impl Message<HandleIncomingPacket> for PlayerClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        mut msg: HandleIncomingPacket,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(blowfish) = self.blowfish.as_mut() {
            blowfish.decrypt(msg.0.as_mut())?;
        }
        let packet = build_client_packet(msg.0)?;
        packet.accept(ctx.actor_ref()).await?;
        Ok(())
    }
}

use crate::controller::GameController;
use crate::cp_factory::build_client_packet;
use crate::movement::MovementState;
use anyhow::{anyhow, bail};
use bytes::BytesMut;
use entities::entities::{character, user};
use entities::DBPool;
use kameo::actor::{ActorId, ActorRef, Spawn, WeakActorRef};
use kameo::error::{ActorStopReason, PanicError};
use kameo::message::{Context, Message};
use kameo::Actor;
use l2_core::crypt::game::GameClientEncryption;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::game_objects::player::Player;
use l2_core::network::connection::{
    send_delayed_packet, send_packet, send_packet_blocking, ConnectionActor, HandleIncomingPacket,
    HandleOutboundPacket,
};
use l2_core::session::SessionKey;
use l2_core::shared_packets::common::SendablePacket;
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use std::fmt;
use std::fmt::Debug;
use std::future::Future;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::sleep;
use tracing::{error, info, instrument};

type BoxedClosure = Box<
    dyn for<'a> FnOnce(
            &'a mut PlayerClient,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
        + Send,
>;
/// A message that executes an async callback after a specified delay
pub struct DoLater {
    pub delay: Duration,
    /// The async callback function to execute
    pub callback: BoxedClosure,
}

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
    pub controller: Arc<GameController>,
    pub ip: Ipv4Addr,
    blowfish: Option<GameClientEncryption>,
    protocol: Option<i32>,
    status: ClientStatus,
    account_chars: Option<Vec<Player>>,
    selected_char: Option<i32>,
    pub packet_sender: Option<ActorRef<ConnectionActor<Self>>>,
    session_key: Option<SessionKey>,
    user: Option<user::Model>,
    pub movement_state: Option<MovementState>,
}

impl Debug for PlayerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player client")
            .field("ip", &self.ip)
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}
impl PlayerClient {
    pub fn new(ip: Ipv4Addr, controller: Arc<GameController>, db_pool: DBPool) -> Self {
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
            movement_state: None,
        }
    }

    ///
    /// Simple usage:
    ///
    /// ```
    /// Self::do_later(
    ///     ctx.actor_ref(),
    ///     DoLater {
    ///         delay: Duration::from_millis(300),
    ///         callback: Box::new(move |actor: &mut PlayerClient| {
    ///             Box::pin(async move {
    ///                 let clan;
    ///                 {
    ///                     let m = actor.controller.clan_ally_manager.read().await;
    ///                     clan = m.clan_list.get(&1).unwrap().clone();
    ///                 }
    ///                 println!("{clan:?}");
    ///                 actor.send_packet(SkillList::empty()?.buffer).await
    ///             })
    ///         }),
    ///     },
    /// );
    /// ```
    pub fn do_later(actor_ref: ActorRef<Self>, task: DoLater) {
        tokio::spawn(async move {
            sleep(task.delay).await;
            // You can add async work here if needed or just immediately send
            let _ = actor_ref.tell(task).await;
        });
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

    pub fn try_get_selected_char_mut(&mut self) -> anyhow::Result<&mut Player> {
        let selected = self
            .selected_char
            .ok_or_else(|| anyhow::anyhow!("Chars not set, possible cheating"))?;

        let chars = self
            .account_chars
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("No characters loaded"))?;
        chars
            .get_mut(usize::try_from(selected)?)
            .ok_or_else(|| anyhow::anyhow!("Selected character not found"))
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

    #[instrument(skip(self, packet))]
    fn prepare_packet_data(&mut self, packet: impl SendablePacket) -> anyhow::Result<BytesMut> {
        let mut data;
        let mut buffer = packet.get_buffer();
        if self.is_encryption_enabled() {
            buffer.write_padding()?;
            data = buffer.take();
            self.encrypt(&mut data)?;
        } else {
            data = buffer.take();
        }
        Ok(data)
    }

    pub async fn send_packet(&mut self, packet: impl SendablePacket) -> anyhow::Result<()> {
        let data = self.prepare_packet_data(packet)?;
        send_packet(self.packet_sender.as_ref(), data.freeze()).await
    }
    #[allow(dead_code)]
    pub async fn send_packet_blocking(
        &mut self,
        packet: impl SendablePacket,
    ) -> anyhow::Result<()> {
        let data = self.prepare_packet_data(packet)?;
        send_packet_blocking(self.packet_sender.as_ref(), data.freeze()).await
    }
    #[allow(dead_code)]
    pub async fn send_packet_later(
        &mut self,
        packet: impl SendablePacket,
        delay: Duration,
    ) -> anyhow::Result<()> {
        let data = self.prepare_packet_data(packet)?;
        send_delayed_packet(self.packet_sender.as_ref(), data.freeze(), delay).await
    }

    /// Start or restart player movement
    pub fn start_movement(
        &mut self,
        dest_x: i32,
        dest_y: i32,
        dest_z: i32,
    ) -> anyhow::Result<(i32, i32, i32)> {
        // Cancel existing movement if any and get current position
        let (current_x, current_y, current_z) = if let Some(mut existing_movement) = self.movement_state.take() {
            existing_movement.cancel_task();
            existing_movement.calculate_current_position()
        } else {
            // Use player's stored position
            let player = self.try_get_selected_char()?;
            (player.get_x(), player.get_y(), player.get_z())
        };

        // Get player speed
        let player = self.try_get_selected_char()?;
        let speed = if player.is_running() {
            player.get_run_speed()
        } else {
            player.get_walk_speed()
        };

        // Create new movement state
        let movement = MovementState::new(
            current_x,
            current_y,
            current_z,
            dest_x,
            dest_y,
            dest_z,
            speed,
        );

        self.movement_state = Some(movement);
        Ok((current_x, current_y, current_z))
    }

    /// Stop current movement and return the current interpolated position
    pub fn stop_movement(&mut self) -> Option<(i32, i32, i32)> {
        if let Some(mut movement) = self.movement_state.take() {
            movement.cancel_task();
            Some(movement.calculate_current_position())
        } else {
            None
        }
    }
}


impl Actor for PlayerClient {
    type Args = (
        Self,
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
    type Error = anyhow::Error;
    fn name() -> &'static str {
        "PlayerClient"
    }
    async fn on_start(args: Self::Args, pl_actor: ActorRef<Self>) -> anyhow::Result<Self> {
        let (mut state, reader, writer) = args;
        info!("Player client {} started: ", state.ip);
        let connection = ConnectionActor::spawn(ConnectionActor::new(
            pl_actor.clone(),
            state.ip,
            reader,
            writer,
            Duration::from_secs(0),
        ));
        connection.wait_for_startup().await;
        pl_actor.link(&connection).await;
        state.packet_sender = Some(connection);
        Ok(state)
    }
    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> anyhow::Result<ControlFlow<ActorStopReason>> {
        error!("Player client {} panicked: {:?}", self.ip, &err);
        if let Some(sender) = self.packet_sender.take() {
            let _ = sender.stop_gracefully().await;
            sender.wait_for_shutdown().await;
        }
        Ok(ControlFlow::Break(ActorStopReason::Panicked(err)))
    }
    async fn on_link_died(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _id: ActorId,
        reason: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        Ok(ControlFlow::Break(reason))
    }
    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> anyhow::Result<()> {
        info!("Disconnecting Client...");
        if let Some(s) = self.packet_sender.as_ref() {
            if s.is_alive() {
                let _ = s.stop_gracefully().await; //ignore errors is it is already dead
            }
            s.wait_for_shutdown().await;
        }
        let Some(user) = self.user.as_ref() else {
            return Ok(());
        };
        if let Ok(player) = self.try_get_selected_char() {
            let res = character::Model::update_char(&self.db_pool, &player.char_model).await;
            if let Err(e) = res {
                error!("Unable to save Player {} state, error: {:?}", self.ip, e);
            }
        }
        self.controller.logout_account(&user.username);
        let packet = match PlayerLogout::new(&user.username) {
            Err(e) => {
                error!("Cannot build logout packet: {}", e);
                //exit function
                return Ok(());
            }
            Ok(p) => p,
        };

        let ls_actor = self.controller.try_get_ls_actor().await?;
        if let Err(err) = ls_actor.tell(packet).await {
            error!(
                "Error while sending logout to login server, cause: {:?}",
                err
            );
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
        packet.accept(ctx.actor_ref().clone()).await?;
        Ok(())
    }
}

impl<P> Message<HandleOutboundPacket<P>> for PlayerClient
where
    P: SendablePacket + Send + 'static,
{
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        msg: HandleOutboundPacket<P>,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.send_packet(msg.packet).await?;
        Ok(())
    }
}

impl Message<DoLater> for PlayerClient {
    type Reply = anyhow::Result<()>;

    async fn handle(&mut self, msg: DoLater, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // Execute the callback with a mutable reference to self
        (msg.callback)(self).await
    }
}

#[derive(Debug)]
pub struct GetCharInfo;

impl Message<GetCharInfo> for PlayerClient {
    type Reply = anyhow::Result<Player>;

    async fn handle(
        &mut self,
        _: GetCharInfo,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.try_get_selected_char()?.clone())
    }
}

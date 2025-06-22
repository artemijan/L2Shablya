use crate::controller::LoginController;
use crate::packet::cp_factory::build_client_message_packet;
use crate::packet::to_client::Init;
use entities::DBPool;
use kameo::prelude::*;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::network::connection::{send_packet, send_packet_blocking, ConnectionActor, HandleIncomingPacket};
use l2_core::session::SessionKey;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, info};
use l2_core::shared_packets::write::SendablePacketBuffer;

#[derive(Clone)]
pub struct LoginClient {
    pub addr: Ipv4Addr,
    pub controller: Arc<LoginController>,
    pub session_id: i32,
    pub db_pool: DBPool,
    encryption: Encryption,
    pub blowfish_key: Vec<u8>,
    pub packet_sender: Option<ActorRef<ConnectionActor<Self>>>,
    pub session_key: SessionKey,
    pub account_name: Option<String>,
    rsa_keypair: ScrambledRSAKeyPair,
    // Rate limiting fields
    last_message_time: Option<Instant>,
    rate_limit_duration: Duration,
}

impl LoginClient {
    pub fn new(addr: Ipv4Addr, controller: Arc<LoginController>, db_pool: DBPool) -> Self {
        let blowfish_key = generate_blowfish_key(None);
        let encryption = Encryption::new(&blowfish_key.clone());
        let rsa_keypair = controller.get_random_rsa_key_pair().clone();
        Self {
            addr,
            blowfish_key,
            session_key: SessionKey::new(),
            account_name: None,
            encryption,
            controller,
            packet_sender: None,
            session_id: LoginController::generate_session_id(),
            db_pool,
            rsa_keypair,
            // Initialize rate limiting with default values
            last_message_time: None,
            rate_limit_duration: Duration::from_millis(300), // 500ms between messages
        }
    }
    // Check if a player is allowed to send a packet based on rate limits
    fn can_accept_packet(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_message_time {
            if now.duration_since(last_time) < self.rate_limit_duration {
                return false; // Too soon, a rate limit is active
            }
        }
        // Update the last message time
        self.last_message_time = Some(now);
        true
    }
    pub async fn send_packet(&self, buffer: SendablePacketBuffer) -> anyhow::Result<()> {
        let data = buffer.take();
        send_packet(self.packet_sender.as_ref(), data.freeze()).await
    }
    pub async fn send_packet_wait_sent(&self, buffer: SendablePacketBuffer) -> anyhow::Result<()> {
        let data = buffer.take();
        send_packet_blocking(self.packet_sender.as_ref(), data.freeze()).await
    }
}

impl Actor for LoginClient {
    type Args = (
        Self,
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, player_actor: ActorRef<Self>) -> anyhow::Result<Self> {
        let (mut state, reader, writer) = args;
        info!("Player {} started", state.addr);
        let connection = ConnectionActor::spawn(ConnectionActor::new(
            player_actor.clone(),
            state.addr,
            reader,
            writer,
            Duration::from_secs(state.controller.get_config().client.timeout.into()),
        ));
        connection.wait_for_startup().await;
        state.packet_sender = Some(connection);
        let init = Init::new(
            state.session_id,
            state.rsa_keypair.get_scrambled_modulus(),
            state.blowfish_key.clone(),
        )?;
        state.send_packet_wait_sent(init.buffer).await?;
        Ok(state)
    }

    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
        info!("[Player {}] stopped", self.addr);
        if let Some(s) = self.packet_sender.as_ref() {
            let _ = s.stop_gracefully().await; //ignore errors is it is already dead
            s.wait_for_shutdown().await;
        }
        Ok(())
    }
    
    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> anyhow::Result<ControlFlow<ActorStopReason>> {
        error!("Login server client {} panicked: {:?}", self.addr, &err);
        Ok(ControlFlow::Break(ActorStopReason::Panicked(err)))
    }
}

impl Message<HandleIncomingPacket> for LoginClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        mut msg: HandleIncomingPacket,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Apply rate limiting
        if !self.can_accept_packet() {
            info!("Rate limited.");
            return Ok(());
        }
        self.encryption.decrypt(msg.0.as_mut())?;
        let packet = build_client_message_packet(msg.0, &self.rsa_keypair)?;
        packet.accept(ctx.actor_ref()).await?;
        Ok(())
    }
}

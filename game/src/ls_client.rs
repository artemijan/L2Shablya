use crate::controller::Controller;
use crate::lsp_factory::build_ls_packet;
use anyhow::bail;
use entities::DBPool;
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::{ActorStopReason, PanicError};
use kameo::message::{Context, Message};
use kameo::Actor;
use l2_core::crypt::login::Encryption;
use l2_core::network::connection::{
    ConnectionActor, HandleIncomingPacket,
};
use l2_core::shared_packets::ls_2_gs::PlayerAuthResponse;
use l2_core::traits::ServerToServer;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tracing::{error, info, instrument};

#[derive(Debug)]
pub enum LSMessages {
    PlayerAuthResponse(PlayerAuthResponse),
}
pub struct LoginServerClient {
    pub db_pool: DBPool,
    ip: Ipv4Addr,
    pub controller: Arc<Controller>,
    blowfish: Encryption,
    pub packet_sender: Option<ActorRef<ConnectionActor<Self>>>,
    pub pending_requests: HashMap<String, oneshot::Sender<LSMessages>>,
}

impl fmt::Debug for LoginServerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LS client")
            .field("ip", &self.ip)
            .finish_non_exhaustive()
    }
}
impl LoginServerClient {
    pub fn new(ip: Ipv4Addr, controller: Arc<Controller>, db_pool: DBPool) -> Self {
        let blowfish = Encryption::from_u8_key(controller.get_cfg().blowfish_key.as_bytes());
        Self {
            db_pool,
            ip,
            controller,
            blowfish,
            packet_sender: None,
            pending_requests: HashMap::new(),
        }
    }
    pub fn set_blowfish(&mut self, blowfish: Encryption) {
        self.blowfish = blowfish;
    }
}

impl Actor for LoginServerClient {
    type Args = (
        Self,
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
    type Error = anyhow::Error;
    fn name() -> &'static str {
        "LSClient"
    }
    async fn on_start(args: Self::Args, ls_actor: ActorRef<Self>) -> anyhow::Result<Self> {
        let (mut state, reader, writer) = args;
        info!("LS client {} started: ", state.ip);
        let connection = ConnectionActor::spawn(ConnectionActor::new(
            ls_actor.clone(),
            state.ip,
            reader,
            writer,
            Duration::from_secs(0),
        ));
        connection.wait_for_startup().await;
        state.packet_sender = Some(connection);
        state.controller.set_ls_actor(ls_actor).await;
        Ok(state)
    }
    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> anyhow::Result<ControlFlow<ActorStopReason>> {
        error!("LS client {} panicked: {:?}", self.ip, &err);
        if let Some(sender) = self.packet_sender.take() {
            let _ = sender.stop_gracefully().await;
            sender.wait_for_shutdown().await;
        }
        Ok(ControlFlow::Break(ActorStopReason::Panicked(err)))
    }
    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> anyhow::Result<()> {
        if let Some(s) = self.packet_sender.as_ref() {
            let _ = s.stop_gracefully().await; //ignore errors is it is already dead
            s.wait_for_shutdown().await;
        }
        Ok(())
    }
}
impl Message<HandleIncomingPacket> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        mut msg: HandleIncomingPacket,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.blowfish.decrypt(msg.0.as_mut())?;
        if !Encryption::verify_checksum(msg.0.as_ref()) {
            bail!("Can not verify check sum.")
        }
        let packet = build_ls_packet(msg.0)?;
        packet.accept(ctx.actor_ref()).await?;
        Ok(())
    }
}
impl ServerToServer for LoginServerClient {
    fn get_packet_sender(&self) -> Option<&ActorRef<ConnectionActor<Self>>> {
        self.packet_sender.as_ref()
    }
    fn get_blowfish(&self) -> &Encryption {
        &self.blowfish
    }
}

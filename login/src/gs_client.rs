use crate::controller::LoginController;
use crate::enums;
use crate::packet::gs_factory::build_gs_packet;
use anyhow::{anyhow, bail};
use entities::DBPool;
use kameo::actor::{ActorId, ActorRef, Spawn, WeakActorRef};
use kameo::error::{ActorStopReason, PanicError};
use kameo::message::{Context, Message};
use kameo::Actor;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::network::connection::{ConnectionActor, HandleIncomingPacket};
use l2_core::shared_packets::common::GSLoginFail;
use l2_core::shared_packets::gs_2_ls::ReplyChars;
use l2_core::shared_packets::ls_2_gs::InitLS;
use l2_core::traits::ServerToServer;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tracing::{error, info};

pub enum GSMessages {
    ReplyChars(ReplyChars),
}
pub struct GameServerClient {
    pub lc: Arc<LoginController>,
    pub db_pool: DBPool,
    pub ip: Ipv4Addr,
    pub key_pair: ScrambledRSAKeyPair,
    blowfish: Encryption,
    //this is used to store the pending requests from the login client;
    // once GS sends a response via TCP, we can answer the request
    pub pending_requests: HashMap<String, oneshot::Sender<GSMessages>>,
    pub packet_sender: Option<ActorRef<ConnectionActor<Self>>>,
    pub connection_state: enums::GS,
    pub server_id: Option<u8>,
}
impl GameServerClient {
    pub fn new(ip: Ipv4Addr, lc: Arc<LoginController>, db_pool: DBPool) -> Self {
        let cfg = lc.get_config();
        Self {
            db_pool,
            ip,
            packet_sender: None,
            key_pair: lc.get_random_rsa_key_pair().clone(), // we have to clone it as we need ownership
            blowfish: Encryption::from_u8_key(cfg.blowfish_key.as_bytes()),
            pending_requests: HashMap::new(),
            connection_state: enums::GS::Initial,
            lc,
            server_id: None,
        }
    }
    pub async fn set_connection_state(&mut self, state: &enums::GS) -> anyhow::Result<()> {
        if let Err(err) = self.connection_state.transition_to(state) {
            let err_msg = format!("Connection state transition failed {err:?}");
            self.send_packet(GSLoginFail::new(err)?).await?;
            bail!(err_msg);
        }
        Ok(())
    }
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) -> anyhow::Result<()> {
        self.blowfish = Encryption::try_from_u8_key(new_bf_key)?;
        Ok(())
    }
    pub fn try_get_server_id(&self) -> anyhow::Result<u8> {
        self.server_id
            .ok_or_else(|| anyhow!("Possible cheating: No server ID set"))
    }
}
impl Actor for GameServerClient {
    type Args = (
        Self,
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, gs_actor: ActorRef<Self>) -> anyhow::Result<Self> {
        let (mut state, reader, writer) = args;
        info!("GS {} started: ", state.ip);
        #[cfg(not(test))]
        {
            state.connection_state = enums::GS::Connected;
        }
        let connection = ConnectionActor::spawn(ConnectionActor::new(
            gs_actor.clone(),
            state.ip,
            reader,
            writer,
            Duration::from_secs(0),
        ));
        connection.wait_for_startup().await;
        gs_actor.link(&connection).await;
        state.packet_sender = Some(connection);
        let init_packet = InitLS::new(state.key_pair.get_modulus());
        state.send_packet(init_packet).await?;
        Ok(state)
    }

    async fn on_panic(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> anyhow::Result<ControlFlow<ActorStopReason>> {
        error!("GS client {} panicked: {:?}", self.ip, &err);
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
        _: WeakActorRef<Self>,
        _: ActorStopReason,
    ) -> Result<(), Self::Error> {
        info!(
            "Game server disconnected: ID ({:})",
            self.server_id.unwrap_or_default()
        );
        if let Some(s) = self.packet_sender.as_ref() {
            if s.is_alive() {
                let _ = s.stop_gracefully().await; //ignore errors is it is already dead
            }
            s.wait_for_shutdown().await;
        }
        if let Some(server_id) = self.server_id {
            self.lc.remove_gs(server_id);
            self.lc.gs_actors.remove(&server_id);
            self.lc.remove_all_gs_players(server_id);
            self.server_id = None;
        }
        Ok(())
    }
}

impl Message<HandleIncomingPacket> for GameServerClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        mut msg: HandleIncomingPacket,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.blowfish.decrypt(msg.0.as_mut())?;
        if !Encryption::verify_checksum(msg.0.as_ref()) {
            bail!("Can not verify check sum.")
        }
        let packet = build_gs_packet(msg.0)?;
        packet.accept(ctx.actor_ref().clone()).await?;
        Ok(())
    }
}
impl ServerToServer for GameServerClient {
    fn get_packet_sender(&self) -> Option<&ActorRef<ConnectionActor<Self>>> {
        self.packet_sender.as_ref()
    }

    fn get_blowfish(&self) -> &Encryption {
        &self.blowfish
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::game_server::GSInfo;
    use crate::dto::player;
    use crate::test_utils::test::{spawn_custom_gs_client_actor, spawn_gs_client_actor};
    use entities::test_factories::factories::user_factory;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use test_utils::utils::{get_test_db, test_hex_id};
    use tokio::io::split;

    #[tokio::test]
    async fn test_shutdown_properly() {
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        user_factory(&db_pool, |mut u| {
            u.username = acc.clone();
            u.access_level = 0;
            u
        })
        .await;
        let (_client, server) = tokio::io::duplex(1024);
        let (_client2, server2) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let (cr, cw) = split(server2);
        let ip = Ipv4Addr::LOCALHOST;
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());
        gs_client.server_id = Some(1);
        gs_client
            .lc
            .register_gs(
                GSInfo::new(
                    1,
                    true,
                    false,
                    9106,
                    true,
                    1,
                    false,
                    1,
                    0,
                    false,
                    5000,
                    test_hex_id(),
                    &["192.168.0.100/8".to_string(), "192.168.0.0".to_string()],
                )
                .unwrap(),
            )
            .unwrap();
        let another_gs_actor = spawn_gs_client_actor(lc.clone(), db_pool.clone(), cr, cw).await;
        gs_client.lc.gs_actors.insert(1, another_gs_actor);
        gs_client.lc.players.insert(
            acc.clone(),
            player::Info {
                game_server: Some(1),
                account_name: acc,
                ..player::Info::default()
            },
        );

        let gs_actor =
            spawn_custom_gs_client_actor(lc.clone(), db_pool.clone(), r, w, Some(gs_client)).await;
        gs_actor.stop_gracefully().await.unwrap();
        gs_actor.wait_for_shutdown().await;
        assert_eq!(lc.gs_actors.len(), 0);
        assert_eq!(lc.game_servers.len(), 0);
        assert_eq!(lc.players.len(), 0);
    }
}

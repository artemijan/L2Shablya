use crate::ls_client::LSMessages;
use crate::packets::to_client::{CharSelectionInfo, PlayerLoginResponse};
use crate::pl_client::{ClientStatus, PlayerClient};
use anyhow::bail;
use bytes::BytesMut;
use entities::dao::item::LocType;
use entities::entities::{character, user};
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::game_objects::player::Player;
use l2_core::session::SessionKey;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
pub struct AuthLogin {
    pub login_name: String,
    pub play_key_1: i32,
    pub play_key_2: i32,
    pub login_key_1: i32,
    pub login_key_2: i32,
    pub buffer: SendablePacketBuffer,
}

impl ReadablePacket for AuthLogin {
    const PACKET_ID: u8 = 0x2B;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let login_name = buffer.read_c_utf16le_string()?.to_lowercase();
        let play_key_2 = buffer.read_i32()?; // wtf? the client decided to send first play_key_2 and not play_key_1
        let play_key_1 = buffer.read_i32()?;
        let login_key_1 = buffer.read_i32()?;
        let login_key_2 = buffer.read_i32()?;
        Ok(Self {
            login_name,
            play_key_1,
            play_key_2,
            login_key_1,
            login_key_2,
            buffer: SendablePacketBuffer::empty(),
        })
    }
}

impl AuthLogin {
    /// Finalizes authentication by updating user status and sending necessary packets.
    async fn authenticate_user(
        &self,
        handler: &mut PlayerClient,
        session_key: SessionKey,
    ) -> anyhow::Result<()> {
        // Notify that the player is in-game
        handler
            .controller
            .try_get_ls_actor()
            .await?
            .tell(PlayerInGame::new(&[self.login_name.clone()])?)
            .await?;

        // Update handler status
        handler.set_status(ClientStatus::Authenticated);
        handler.set_session_key(session_key);

        // Send success response
        handler.send_packet(PlayerLoginResponse::ok()?).await?;

        // Fetch user characters from the database
        let characters = character::Model::get_with_items_and_vars(
            &handler.db_pool,
            &self.login_name,
            LocType::Paperdoll,
        )
        .await?;

        let players = characters
            .into_iter()
            .map(|(ch, items)| {
                let template = handler
                    .controller
                    .class_templates
                    .try_get_template(ch.class_id)?;
                Ok(Player::new(ch, items, template.clone()))
            })
            .collect::<anyhow::Result<Vec<Player>>>()?;

        // Prepare a character selection packet
        let char_selection = CharSelectionInfo::new(
            &self.login_name,
            self.play_key_1,
            &handler.controller,
            &players,
        )?;

        // Update handler with retrieved data
        handler.set_account_chars(players);
        let user = user::Model::find_by_username(&handler.db_pool, &self.login_name).await?;
        handler.set_user(user);

        // Send character selection info
        handler.send_packet(char_selection).await?;

        Ok(())
    }
}
impl Message<AuthLogin> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: AuthLogin,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        // Check if the user is already authenticated
        if self.get_user().is_some() {
            return Ok(());
        }
        let _cfg = self.controller.get_cfg();

        // Ensure the protocol is set and login_name is not empty
        if self.get_protocol().is_none() || msg.login_name.is_empty() {
            bail!("Protocol version not set");
        }

        // Try to add the user to the online accounts
        if self
            .controller
            .add_online_account(&msg.login_name, Some(ctx.actor_ref().clone()))
            .is_some()
        {
            self.controller.logout_account(&msg.login_name);
            bail!("Account already in game: {}", msg.login_name);
        }

        let session_key = SessionKey {
            play_ok1: msg.play_key_1,
            play_ok2: msg.play_key_2,
            login_ok1: msg.login_key_1,
            login_ok2: msg.login_key_2,
        };

        // Send authentication request
        let auth_request = PlayerAuthRequest::new(&msg.login_name, session_key.clone())?;
        let actor = self.controller.try_get_ls_actor().await?;
        match actor.ask(auth_request).await {
            Ok(response_future) => match response_future.await {
                Ok(LSMessages::PlayerAuthResponse(r)) if r.is_ok => {
                    return msg.authenticate_user(self, session_key).await;
                }
                Ok(LSMessages::PlayerAuthResponse(r)) => {
                    // Handle auth response that is not OK
                    tracing::warn!("Authentication failed: {:?}", r);
                }
                Err(e) => {
                    // Handle error in the second await
                    error!("Failed to await response future: {:?}", e);
                }
            },
            Err(e) => {
                // Handle error from actor.ask
                error!("Failed to send ask to actor: {:?}", e);
            }
        }

        // Authentication failed
        self.send_packet(PlayerLoginResponse::fail(
            PlayerLoginResponse::SYSTEM_ERROR_LOGIN_LATER,
        )?)
        .await?;
        self.controller.logout_account(&msg.login_name);
        bail!("Login failed: {}", msg.login_name);
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::GameController;
    use crate::ls_client::LoginServerClient;
    use crate::packets::from_client::auth::AuthLogin;
    use crate::pl_client::{ClientStatus, PlayerClient};
    use crate::test_utils::test::{
        get_gs_config, spawn_custom_ls_client_actor, spawn_custom_player_client_actor,
    };
    use entities::test_factories::factories::user_factory;
    use l2_core::shared_packets::ls_2_gs::PlayerAuthResponse;
    use l2_core::shared_packets::write::SendablePacketBuffer;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use std::time::Duration;
    use test_utils::utils::get_test_db;
    use tokio::io::split;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_auth() {
        // Create a listener on a local port
        let (_client, server) = tokio::io::duplex(1024);
        let (_ls_client_stream, ls_server) = tokio::io::duplex(1024);
        let cfg = get_gs_config();
        let pool = get_test_db().await;
        let _ = user_factory(&pool, |mut u| {
            u.username = "test".to_owned();
            u
        })
        .await;
        let controller = Arc::new(GameController::from_config(Arc::new(cfg)));
        let (ls_r, ls_w) = split(ls_server);
        let ls_client =
            LoginServerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());
        let auth = AuthLogin {
            login_name: "test".to_string(),
            play_key_1: 0,
            play_key_2: 0,
            login_key_1: 0,
            login_key_2: 0,
            buffer: SendablePacketBuffer::empty(),
        };
        let ls_actor = spawn_custom_ls_client_actor(
            controller.clone(),
            pool.clone(),
            ls_r,
            ls_w,
            Some(ls_client),
        )
        .await;
        controller.set_ls_actor(ls_actor.clone()).await;
        let (r, w) = split(server);
        let mut player_client =
            PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());
        player_client.set_status(ClientStatus::Connected);
        player_client
            .set_protocol(110)
            .expect("Protocol version is wrong");
        let player_actor =
            spawn_custom_player_client_actor(controller, pool, r, w, Some(player_client)).await;
        //--> auth login
        let pb = PlayerAuthResponse::new(&auth.login_name, true);
        tokio::spawn(async move {
            //todo I don't like it, maybe send encrypted packet to _ls_client_stream instead?
            sleep(Duration::from_millis(100)).await;
            ls_actor
                .tell(pb)
                .await
                .expect("Failed to send packet to the login server");
        });
        let res = player_actor.ask(auth).await;
        assert!(res.is_ok());
    }
}

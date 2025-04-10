use std::sync::Arc;

use crate::client_thread::{ClientHandler, ClientStatus};
use crate::controller::Controller;
use crate::ls_thread::LoginHandler;
use crate::packets::to_client::{CharSelectionInfo, PlayerLoginResponse};
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::dao::item::LocType;
use entities::entities::{character, user};
use l2_core::game_objects::player::Player;
use l2_core::session::SessionKey;
use l2_core::shared_packets::common::{PacketType, ReadablePacket};
use l2_core::shared_packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use l2_core::traits::handlers::PacketSender;
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
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
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let login_name = buffer.read_c_utf16le_string()?.to_lowercase();
        let play_key_2 = buffer.read_i32()?; // wtf? client decided to send first play_key_2 and not play_key_1
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
        handler: &mut ClientHandler,
        controller: &Arc<Controller>,
        session_key: SessionKey,
    ) -> anyhow::Result<()> {
        // Notify that player is in-game
        controller
            .message_broker
            .notify(
                LoginHandler::HANDLER_ID,
                Box::new(PlayerInGame::new(&[self.login_name.clone()])?),
            )
            .await?;

        // Update handler status
        handler.set_status(ClientStatus::Authenticated);
        handler.set_session_key(session_key);

        // Send success response
        handler
            .send_packet(Box::new(PlayerLoginResponse::ok()?))
            .await?;

        // Fetch user characters from the database
        let db_pool = handler.get_db_pool().clone();
        let characters = character::Model::get_with_items_and_vars(
            &db_pool,
            &self.login_name,
            LocType::Paperdoll,
        )
        .await?;
        
        let players= characters.into_iter().map(|(ch, items)|{
            Player::new(ch, items)
        }).collect::<anyhow::Result<Vec<Player>>>()?;

        // Prepare character selection packet
        let char_selection =
            CharSelectionInfo::new(&self.login_name, self.play_key_1, controller, &players)?;

        // Update handler with retrieved data
        handler.set_account_chars(players);
        let user = user::Model::find_by_username(&db_pool, &self.login_name).await?;
        handler.set_user(user);

        // Send character selection info
        handler.send_packet(Box::new(char_selection)).await?;

        Ok(())
    }
}

#[async_trait]
impl HandleablePacket for AuthLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        // Check if user is already authenticated
        if handler.get_user().is_some() {
            return Ok(());
        }
        let controller = handler.get_controller().clone();
        let _cfg = controller.get_cfg();

        // Ensure protocol is set and login_name is not empty
        if handler.get_protocol().is_none() || self.login_name.is_empty() {
            bail!("Protocol version not set");
        }

        // Try to add the user to the online accounts
        if controller
            .add_online_account(self.login_name.clone())
            .is_some()
        {
            controller.logout_account(&self.login_name);
            bail!("Account already in game: {}", self.login_name);
        }

        let session_key = SessionKey {
            play_ok1: self.play_key_1,
            play_ok2: self.play_key_2,
            login_ok1: self.login_key_1,
            login_ok2: self.login_key_2,
        };

        // Send authentication request
        let auth_request = PlayerAuthRequest::new(&self.login_name, session_key.clone())?;
        let response = controller
            .message_broker
            .send_message(
                LoginHandler::HANDLER_ID,
                &self.login_name,
                Box::new(auth_request),
            )
            .await?;

        // Handle authentication response
        if let Some((_, PacketType::PlayerAuthResp(p))) = response {
            if p.is_ok {
                return self
                    .authenticate_user(handler, &controller, session_key)
                    .await;
            }
        }

        // Authentication failed
        handler
            .send_packet(Box::new(PlayerLoginResponse::fail(
                PlayerLoginResponse::SYSTEM_ERROR_LOGIN_LATER,
            )?))
            .await?;
        controller.logout_account(&self.login_name);
        bail!("Login failed: {}", self.login_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use crate::packets::from_client::auth::AuthLogin;
    use crate::packets::to_client::PlayerLoginResponse;
    use crate::tests::{get_gs_config, TestPacketSender};
    use entities::test_factories::factories::user_factory;
    use l2_core::shared_packets::common::{PacketType, ReadablePacket, SendablePacket};
    use l2_core::shared_packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
    use l2_core::shared_packets::ls_2_gs::PlayerAuthResponse;
    use l2_core::shared_packets::read::ReadablePacketBuffer;
    use ntest::timeout;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    use tokio::sync::Mutex;
    use l2_core::traits::handlers::PacketHandler;

    #[tokio::test]
    #[timeout(3000)]
    async fn test_auth() {
        // Create a listener on a local port
        let (mut client, server) = tokio::io::duplex(1024);
        let (login_client, mut login_server) = tokio::io::duplex(1024);
        let cfg = get_gs_config();
        let pool = get_test_db().await;
        let _ = user_factory(&pool, |mut u| {
            u.username = "test".to_owned();
            u
        })
        .await;
        let controller = Arc::new(Controller::from_config(Arc::new(cfg)));
        let test_packet_sender = Arc::new(TestPacketSender {
            writer: Arc::new(Mutex::new(login_client)),
        });
        controller
            .message_broker
            .register_packet_handler(LoginHandler::HANDLER_ID, test_packet_sender);
        let db_pool = pool.clone();
        let contr = controller.clone();
        tokio::spawn(async move {
            let ip = Ipv4Addr::new(127, 0, 0, 1);
            let (r, w) = split(server);
            let mut ch = ClientHandler::new(r, w, ip, db_pool, contr);
            ch.set_status(ClientStatus::Connected);
            ch.set_protocol(110).unwrap();
            ch.handle_client().await
        });
        //--> auth login
        let mut auth = AuthLogin::new().unwrap();
        client.write_all(auth.get_bytes(false)).await.unwrap();
        let mut auth_login_packet = [0; 29];
        // <-- Try login on login server
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

        // --> login ok
        let auth_ok_packet = PlayerAuthResponse::new("test", true);
        controller.message_broker.respond_to_message(
            Some(LoginHandler::HANDLER_ID),
            "test",
            PacketType::PlayerAuthResp(auth_ok_packet),
        );
        // <-- Players in game (received by login server)
        let mut player_in_game = [0; 15];
        login_server.read_exact(&mut player_in_game).await.unwrap();
        let pig = PlayerInGame::read(&player_in_game[2..]).unwrap();
        assert_eq!(pig.accounts, ["test"]);
        // <-- Player auth ok
        let mut auth_ok_resp = [0; 11];
        client.read_exact(&mut auth_ok_resp).await.unwrap();
        let mut buffer = ReadablePacketBuffer::new(&auth_ok_resp[2..]);
        let p_id = buffer.read_byte().unwrap();
        let is_ok = buffer.read_i32().unwrap();
        let reason = buffer.read_u32().unwrap();
        assert_eq!(PlayerLoginResponse::PACKET_ID, p_id);
        assert_eq!(is_ok, -1);
        assert_eq!(reason, 0);
    }
}

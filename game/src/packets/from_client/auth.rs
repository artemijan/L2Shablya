use crate::client_thread::{ClientHandler, ClientStatus};
use crate::ls_thread::LoginHandler;
use crate::packets::to_client::{CharSelectionInfo, PlayerLoginResponse};
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use tracing::info;
use entities::dao::item::LocType;
use entities::entities::{character, user};
use l2_core::session::SessionKey;
use l2_core::shared_packets::common::{PacketType, ReadablePacket};
use l2_core::shared_packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct AuthLogin {
    pub login_name: String,
    pub play_key_1: i32,
    pub play_key_2: i32,
    pub login_key_1: i32,
    pub login_key_2: i32,
}

impl ReadablePacket for AuthLogin {
    const PACKET_ID: u8 = 0x2B;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        let login_name = buffer.read_string().to_lowercase();
        let play_key_2 = buffer.read_i32(); // wtf? client decided to send first play_key_2 and not play_key_1
        let play_key_1 = buffer.read_i32();
        let login_key_1 = buffer.read_i32();
        let login_key_2 = buffer.read_i32();
        Ok(Self {
            login_name,
            play_key_1,
            play_key_2,
            login_key_1,
            login_key_2,
        })
    }
}

#[async_trait]
impl HandleablePacket for AuthLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = handler.get_controller().clone();
        let _cfg = controller.get_cfg();
        if handler.get_protocol().is_none() || self.login_name.is_empty() {
            bail!("Protocol version not set");
        }
        if handler.user.is_none() {
            if controller
                .add_online_account(self.login_name.clone())
                .is_none()
            {
                let _session_key = SessionKey {
                    play_ok1: self.play_key_1,
                    play_ok2: self.play_key_2,
                    login_ok1: self.login_key_1,
                    login_ok2: self.login_key_2,
                };
                let resp = controller
                    .message_broker
                    .send_message(
                        LoginHandler::HANDLER_ID,
                        &self.login_name,
                        Box::new(PlayerAuthRequest::new(
                            &self.login_name,
                            _session_key.clone(),
                        )?),
                    )
                    .await?;
                match resp {
                    Some((_, PacketType::PlayerAuthResp(p))) if p.is_ok => {
                        controller
                            .message_broker
                            .notify(
                                LoginHandler::HANDLER_ID,
                                Box::new(PlayerInGame::new(&[self.login_name.clone()])?),
                            )
                            .await?;
                        handler.set_status(ClientStatus::Authenticated);
                        handler.set_session_key(_session_key);
                        handler
                            .send_packet(Box::new(PlayerLoginResponse::ok()?))
                            .await?;
                        let db_pool = handler.get_db_pool().clone();
                        let chars = character::Model::get_with_items_and_vars(
                            &db_pool,
                            &self.login_name,
                            LocType::Paperdoll,
                        )
                        .await?;
                        let p = CharSelectionInfo::new(
                            &self.login_name,
                            self.play_key_1,
                            &controller,
                            &chars,
                        )?;
                        handler.set_account_chars(chars);
                        let user =
                            user::Model::find_by_username(&db_pool, &self.login_name).await?;
                        handler.user = Some(user);
                        handler.send_packet(Box::new(p)).await?;
                    }
                    _ => {
                        handler
                            .send_packet(Box::new(PlayerLoginResponse::fail(
                                PlayerLoginResponse::SYSTEM_ERROR_LOGIN_LATER,
                            )?))
                            .await?;
                        controller.logout_account(&self.login_name);
                        bail!("Login failed {}", self.login_name);
                    }
                }
            } else {
                controller.logout_account(&self.login_name);
                bail!("Account already in game {}", self.login_name);
            }
        }
        Ok(())
    }
}

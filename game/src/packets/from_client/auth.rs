use crate::client_thread::{ClientHandler, ClientStatus};
use crate::ls_thread::LoginHandler;
use crate::packets::to_client::{CharSelectionInfo, PlayerLoginResponse};
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::packets::common::{PacketType, ReadablePacket};
use l2_core::packets::error::PacketRun;
use l2_core::packets::gs_2_ls::{PlayerAuthRequest, PlayerInGame};
use l2_core::packets::read::ReadablePacketBuffer;
use l2_core::session::SessionKey;
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
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte(); // packet_id
        let login_name = buffer.read_string().to_lowercase();
        let play_key_2 = buffer.read_i32(); // wtf? client decided to send first play_key_2 and not play_key_1
        let play_key_1 = buffer.read_i32();
        let login_key_1 = buffer.read_i32();
        let login_key_2 = buffer.read_i32();
        Some(Self {
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
    async fn handle(&self, handler: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = handler.get_controller();
        let _cfg = controller.get_cfg();
        if handler.get_protocol().is_none() || self.login_name.is_empty() {
            return Err(PacketRun {
                msg: Some("Protocol version not set".to_string()),
            })?;
        }
        if handler.account_name.is_none() {
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
                        let contr = controller.clone();
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
                        let mut db_pool = handler.get_db_pool().clone();
                        handler
                            .send_packet(Box::new(
                                CharSelectionInfo::new(
                                    &self.login_name,
                                    self.play_key_1,
                                    contr,
                                    &mut db_pool,
                                )
                                .await?,
                            ))
                            .await?;
                        // todo: set char list to handler
                    }
                    _ => {
                        handler
                            .send_packet(Box::new(PlayerLoginResponse::fail(
                                PlayerLoginResponse::SYSTEM_ERROR_LOGIN_LATER,
                            )?))
                            .await?;
                        controller.remove_online_account(&self.login_name);
                        return Err(PacketRun {
                            msg: Some(format!("Login failed {}", self.login_name)),
                        });
                    }
                }
            } else {
                controller.remove_online_account(&self.login_name);
                return Err(PacketRun {
                    msg: Some(format!("Account already in game {}", self.login_name)),
                });
            }
        }
        Ok(())
    }
}

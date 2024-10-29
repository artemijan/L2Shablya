use crate::common::dto::player;
use crate::common::str::Trim;
use crate::database::user::User;
use crate::login_server::client_thread::Client;
use crate::login_server::traits::PacketHandler;
use crate::packet::common::{ClientHandle, PacketResult};
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::to_client::LoginOk;
use crate::packet::PlayerLoginFailReasons;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct RequestAuthLogin {
    pub username: String,
    pub password: String,
}

impl ReadablePacket for RequestAuthLogin {
    fn read(data: &[u8]) -> Option<Self> {
        let part1 = String::from_utf8_lossy(&data[0x4E..0x4E + 50]);
        let part2 = String::from_utf8_lossy(&data[0xCE..0xCE + 14]);
        let part3 = String::from_utf8_lossy(&data[0xDC..0xDC + 16]);
        Some(RequestAuthLogin {
            username: format!("{}{}", part1.trim_all(), part2.trim_all()),
            password: part3.trim_all().to_string(),
        })
    }
}

#[async_trait]
impl ClientHandle for RequestAuthLogin {
    async fn handle(
        &self,
        ch: &mut Client,
    ) -> PacketResult {
        let auto_registration = ch.get_lc().get_config().auto_registration;
        let pool = ch.get_db_pool_mut();
        let mut user_option = User::fetch_by_username(pool, &self.username)
            .await
            .expect("Can not connect to the DB");
        if let Some(user) = user_option {
            if !user.verify_password(&self.password).await {
                return Err(error::PacketRun {
                    msg: Some(format!("Login Fail, tried user: {}", self.username)),
                    response: Some(Box::new(PlayerLogin::new(
                        PlayerLoginFailReasons::ReasonUserOrPassWrong,
                    ))),
                });
            }
        } else if auto_registration{
            user_option = User::new(pool, &self.username, &self.password).await.ok();
            assert!(user_option.is_some(), "Can not create a user {}", self.username);
        }
        ch.account_name = Some(self.username.to_string());
        let player_info = player::Info {
            is_authed: true,
            account_name: self.username.clone(),
            ..Default::default()
        };
        let lc =ch.get_lc(); 
        lc.on_player_login(player_info).await.expect("TODO: handle panic message");
        Ok(Some(Box::new(LoginOk::new(ch.get_session_key()))))
    }
}

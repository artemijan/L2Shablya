use crate::common::str::Trim;
use crate::database::user::User;
use crate::login_server::ls_handler::ClientHandler;
use crate::login_server::PacketHandler;
use crate::packet::common::ClientHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::to_client::LoginOk;
use crate::packet::PlayerLoginFailReasons;
use anyhow::bail;
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
        ch: &mut ClientHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let auto_registration = ch.get_lc().get_config().auto_registration;
        let pool = ch.get_db_pool_mut();
        if let Some(user) = User::fetch_by_username(pool, &self.username).await.unwrap() {
            if user.verify_password(&self.password).await {
                ch.account_name = Some(self.username.to_string());
                return Ok(Some(Box::new(LoginOk::new(ch.get_session_key()))));
            }
        } else if auto_registration {
            if User::new(pool, &self.username, &self.password)
                .await
                .is_ok()
            {
                ch.account_name = Some(self.username.to_string());
                return Ok(Some(Box::new(LoginOk::new(ch.get_session_key()))));
            }
            panic!("Can not create a user {}", self.username);
        }
        Err(error::PacketRun {
            msg: Some(format!("Login Fail, tried user: {}", self.username)),
            response: Some(Box::new(PlayerLogin::new(
                PlayerLoginFailReasons::ReasonUserOrPassWrong,
            ))),
        })
    }
}

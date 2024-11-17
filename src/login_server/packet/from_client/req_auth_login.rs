use crate::login_server::dto::player;
use crate::common::str::Trim;
use crate::database::user::User;
use crate::login_server::client_thread::ClientHandler;
use crate::common::traits::handler::PacketHandler;
use crate::login_server::packet::common::ClientHandle;
use crate::common::packet::error;
use crate::login_server::packet::to_client::ServerList;
use crate::login_server::packet::{login_fail, to_client::LoginOk, PlayerLoginFailReasons};
use async_trait::async_trait;
use crate::common::packet::{PacketResult, ReadablePacket};

#[derive(Clone, Debug)]
pub struct RequestAuthLogin {
    pub username: String,
    pub password: String,
    is_new_auth: bool,
}

impl ReadablePacket for RequestAuthLogin {
    fn read(data: &[u8]) -> Option<Self> {
        let body = &data[..data.len() - 1]; //exclude last byte from calculation
        let mut is_new_auth = false;
        if let Some(val) = data.last() {
            is_new_auth = *val != 0;
        }
        let mut username = String::new();
        let mut password = String::new();
        if is_new_auth {
            let part1 = String::from_utf8_lossy(&body[0x4E..0x4E + 50]);
            let part2 = String::from_utf8_lossy(&body[0xCE..0xCE + 14]);
            username = format!("{}{}", part1.trim_all(), part2.trim_all());
            password = String::from_utf8_lossy(&body[0xDC..0xDC + 16])
                .trim_all()
                .to_string();
        } else {
            username = String::from_utf8_lossy(&body[0x5E..0x5E + 14]).to_string();
            password = String::from_utf8_lossy(&body[0x6C..0x6C + 16])
                .trim_all()
                .to_string();
        }
        Some(Self {
            username,
            password,
            is_new_auth,
        })
    }
}

#[async_trait]
impl ClientHandle for RequestAuthLogin {
    async fn handle(&self, ch: &mut ClientHandler) -> PacketResult {
        let auto_registration = ch.get_controller().get_config().auto_registration;
        let show_license = ch.get_controller().get_config().client.show_licence;
        let pool = ch.get_db_pool_mut();
        let mut user_option = User::fetch_by_username(pool, &self.username)
            .await
            .expect("DB error");
        if let Some(user) = user_option {
            if !user.verify_password(&self.password).await {
                return Err(error::PacketRun {
                    msg: Some(format!("Login Fail, tried user: {}", self.username)),
                    response: Some(Box::new(login_fail::PlayerLogin::new(
                        PlayerLoginFailReasons::ReasonUserOrPassWrong,
                    ))),
                });
            }
        } else if auto_registration {
            user_option = User::new(pool, &self.username, &self.password).await.ok();
            assert!(
                user_option.is_some(),
                "Can not create a user {}",
                self.username
            );
        }
        ch.account_name = Some(self.username.clone());
        let player_info = player::Info {
            is_authed: true,
            session: Some(ch.get_session_key().clone()),
            account_name: self.username.clone(),
            ..Default::default()
        };
        let lc = ch.get_controller();
        lc.on_player_login(player_info).await?;
        if show_license {
            Ok(Some(Box::new(LoginOk::new(ch.get_session_key()))))
        } else {
            let s_list = ServerList::new(ch, &self.username);
            Ok(Some(Box::new(s_list)))
        }
    }
}

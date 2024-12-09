use crate::common::packets::common::{
    HandleablePacket, PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket,
};
use crate::common::packets::error::PacketRun;
use crate::common::str::Trim;
use crate::common::traits::handlers::PacketHandler;
use crate::database::user::User;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::dto::player;
use crate::login_server::packet::to_client::LoginOk;
use crate::login_server::packet::to_client::ServerList;
use async_trait::async_trait;

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
impl HandleablePacket for RequestAuthLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let auto_registration = ch.get_controller().get_config().auto_registration;
        let show_license = ch.get_controller().get_config().client.show_licence;
        let pool = ch.get_db_pool_mut();
        let mut user_option = User::fetch_by_username(pool, &self.username)
            .await
            .expect("DB error");
        if let Some(user) = user_option {
            if !user.verify_password(&self.password).await {
                ch.send_packet(Box::new(PlayerLoginFail::new(
                    PlayerLoginFailReasons::ReasonUserOrPassWrong,
                )))
                .await?;
                return Err(PacketRun {
                    msg: Some(format!("Login Fail, tried user: {}", self.username)),
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
        if let Err(err) = lc.on_player_login(player_info).await {
            let err_msg = format!("Player login failed: {err:?}");
            ch.send_packet(Box::new(PlayerLoginFail::new(err))).await?;
            return Err(PacketRun { msg: Some(err_msg) });
        }
        if show_license {
            ch.send_packet(Box::new(LoginOk::new(ch.get_session_key())))
                .await?;
        } else {
            let s_list = ServerList::new(ch, &self.username);
            ch.send_packet(Box::new(s_list)).await?;
        }
        Ok(())
    }
}

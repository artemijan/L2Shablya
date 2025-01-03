use crate::client_thread::ClientHandler;
use crate::dto::player;
use crate::packet::to_client::LoginOk;
use crate::packet::to_client::ServerList;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use entities::entities::user;
use l2_core::hash_password;
use l2_core::packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::packets::error::PacketRun;
use l2_core::str::Trim;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use sea_orm::{ActiveModelTrait, ActiveValue};

#[derive(Clone, Debug)]
#[allow(unused)]
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
        let username: String;
        let password: String;
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
            is_new_auth
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
        let user_option = user::Model::find_some_by_username(pool, &self.username).await?;

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
            let password_hash = hash_password(&self.password).await?;
            let user_record = user::ActiveModel {
                id: ActiveValue::NotSet,
                username: ActiveValue::Set(self.username.clone()),
                password: ActiveValue::Set(password_hash),
                access_level: ActiveValue::Set(0),
                ban_duration: ActiveValue::NotSet,
                ban_ip: ActiveValue::NotSet,
            };
            user_record.save(pool).await?;
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

use crate::client_thread::ClientHandler;
use crate::dto::player;
use crate::packet::to_client::LoginOk;
use crate::packet::to_client::ServerList;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::entities::user;
use l2_core::hash_password;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::str::Trim;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use sea_orm::{ActiveModelTrait, ActiveValue};
use std::fmt::Debug;


#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestAuthLogin {
    pub username: String,
    pub password: String,
    is_new_auth: bool,
    pub is_cmd_login: bool,
}

impl ReadablePacket for RequestAuthLogin {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let is_cmd_login = false;
        let (username, password, is_new_auth) = read_bytes(data);
        Ok(Self {
            username,
            password,
            is_new_auth,
            is_cmd_login,
        })
    }
}
#[async_trait]
impl HandleablePacket for RequestAuthLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        let cfg = ch.get_controller().get_config();
        let show_license = cfg.client.show_licence;
        let pool = ch.get_db_pool();
        let user_option = user::Model::find_some_by_username(pool, &self.username).await?;
        if let Some(user) = user_option {
            if !user.verify_password(&self.password).await {
                ch.send_packet(Box::new(PlayerLoginFail::new(
                    PlayerLoginFailReasons::ReasonUserOrPassWrong,
                )?))
                .await?;
                bail!(format!("Login Fail, tried user: {}", &self.username));
            }
        } else if cfg.client.auto_create_accounts {
            let password_hash = hash_password(&self.password).await?;
            let user_record = user::ActiveModel {
                id: ActiveValue::NotSet,
                username: ActiveValue::Set(self.username.to_string()),
                password: ActiveValue::Set(password_hash),
                access_level: ActiveValue::Set(0),
                ban_duration: ActiveValue::NotSet,
                ban_ip: ActiveValue::NotSet,
            };
            user_record.save(pool).await?;
        }

        ch.account_name = Some(self.username.to_string());
        let player_info = player::Info {
            is_authed: true,
            session: Some(ch.get_session_key().clone()),
            account_name: self.username.to_string(),
            ..Default::default()
        };

        let lc = ch.get_controller();
        if let Err(err) = lc.on_player_login(player_info).await {
            let err_msg = format!("Player login failed: {err:?}");
            ch.send_packet(Box::new(PlayerLoginFail::new(err)?)).await?;
            bail!(err_msg);
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

pub fn read_bytes(data: &[u8]) -> (String, String, bool) {
    let mut is_new_auth = false;
    if data.len() >= 256 {
        is_new_auth = true;
    }
    let username: String;
    let password: String;
    if is_new_auth {
        let part1 = String::from_utf8_lossy(&data[0x4E..0x4E + 50]);
        let part2 = String::from_utf8_lossy(&data[0xCE..0xCE + 14]);
        username = format!("{}{}", part1.trim_all(), part2.trim_all());
        password = String::from_utf8_lossy(&data[0xDC..0xDC + 16])
            .trim_all()
            .to_string();
    } else {
        username = String::from_utf8_lossy(&data[0x5E..0x5E + 14])
            .trim_all()
            .to_string();
        password = String::from_utf8_lossy(&data[0x6C..0x6C + 16])
            .trim_all()
            .to_string();
    }
    (username, password, is_new_auth)
}

#[cfg(test)]
mod tests {
    use crate::packet::from_client::RequestAuthLogin;
    use l2_core::shared_packets::common::ReadablePacket;

    #[tokio::test]
    async fn test_read_bytes_login() {
        let login_bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 52, 0, 0, 97, 100, 109, 105, 110, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 97, 100, 109, 105, 110, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1,
        ];
        let p1 = RequestAuthLogin::read(&login_bytes).unwrap();
        assert!(!p1.is_cmd_login);
        assert!(p1.is_new_auth);
        assert_eq!(p1.username, "admin");
        assert_eq!(p1.password, "admin");
    }
}

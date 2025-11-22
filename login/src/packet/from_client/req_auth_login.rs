use crate::dto::player;
use crate::login_client::LoginClient;
use crate::packet::to_client::LoginOk;
use crate::packet::to_client::ServerList;
use anyhow::bail;
use bytes::BytesMut;
use entities::entities::user;
use kameo::message::{Context, Message};
use l2_core::hash_password;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::str::Trim;
use sea_orm::{ActiveModelTrait, ActiveValue};
use std::fmt::Debug;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct RequestAuthLogin {
    pub username: String,
    pub password: String,
}
impl Message<RequestAuthLogin> for LoginClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: RequestAuthLogin,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let cfg = self.controller.get_config();
        let show_license = cfg.client.show_licence;
        let user_option = user::Model::find_some_by_username(&self.db_pool, &msg.username).await?;
        if let Some(user) = user_option {
            if !user.verify_password(&msg.password).await {
                self.send_packet(
                    PlayerLoginFail::new(PlayerLoginFailReasons::ReasonUserOrPassWrong)?,
                )
                .await?;
                bail!(format!("Login Fail, tried user: {}", &msg.username));
            }
        } else if cfg.client.auto_create_accounts {
            if msg.username.is_empty() || msg.password.is_empty() {
                bail!("Username or password is empty");
            }
            let password_hash = hash_password(&msg.password).await?;
            let user_record = user::ActiveModel {
                id: ActiveValue::NotSet,
                username: ActiveValue::Set(msg.username.clone()),
                password: ActiveValue::Set(password_hash),
                access_level: ActiveValue::Set(0),
                ban_duration: ActiveValue::NotSet,
                ban_ip: ActiveValue::NotSet,
            };
            user_record.save(&self.db_pool).await?;
        } else {
            self.send_packet(
                PlayerLoginFail::new(PlayerLoginFailReasons::ReasonUserOrPassWrong)?,
            )
            .await?;
            bail!("User not found, and auto creation of accounts is disabled.");
        }

        self.account_name = Some(msg.username.clone());
        let player_info = player::Info {
            player_actor: Some(ctx.actor_ref().clone()),
            session: Some(self.session_key.clone()),
            account_name: msg.username.clone(),
            ..Default::default()
        };

        if let Err(err) = self.controller.on_player_login(player_info).await {
            let err_msg = format!("Player login failed: {err:?}");
            self.send_packet(PlayerLoginFail::new(err)?).await?;
            bail!(err_msg);
        }

        if show_license {
            self.send_packet(LoginOk::new(&self.session_key))
                .await?;
        } else {
            let s_list = ServerList::new(self, &msg.username);
            self.send_packet(s_list).await?;
        }
        Ok(())
    }
}
impl ReadablePacket for RequestAuthLogin {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let (username, password) = read_bytes(&data);
        Ok(Self { username, password })
    }
}

pub fn read_bytes(data: &BytesMut) -> (String, String) {
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
    (username, password)
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::packet::from_client::RequestAuthLogin;
    use crate::test_utils::test::{spawn_login_client_actor, GetState};
    use bytes::BytesMut;
    use entities::entities::user;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::shared_packets::common::ReadablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

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
        let p1 = RequestAuthLogin::read(BytesMut::from(&login_bytes[..])).unwrap();
        assert_eq!(p1.username, "admin");
        assert_eq!(p1.password, "admin");
    }
    #[tokio::test]
    async fn test_handle_login() {
        let packet = RequestAuthLogin {
            username: "admin".to_string(),
            password: "admin".to_string(),
        };
        let db_pool = get_test_db().await;
        user_factory(&db_pool, |mut u| {
            u.username = "admin".to_owned();
            u.password = "$argon2id$v=19$m=19456,t=2,p=1$OnSjOZTt6Or9MxtqrcrGhw$GAY7oGKMMAQbd6tvWB96IjA6yxvZy2PMD2MEpHbmWS0".to_owned();
            u
        })
        .await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool, r, w).await;
        let result = player_actor.ask(packet).await;
        let state = player_actor.ask(GetState).await.unwrap();
        assert!(result.is_ok());
        assert_eq!(state.account_name, Some("admin".to_string()));
    }
    #[tokio::test]
    async fn test_handle_login_auto_create_account() {
        let packet = RequestAuthLogin {
            username: "test".to_string(),
            password: "test".to_string(),
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = true;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool.clone(), r, w).await;
        let result = player_actor.ask(packet).await;
        let state = player_actor.ask(GetState).await.unwrap();
        let user = user::Model::find_some_by_username(&db_pool, "test")
            .await
            .unwrap();
        assert!(result.is_ok());
        assert_eq!(state.account_name, Some("test".to_string()));
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "test");
    }

    #[tokio::test]
    async fn test_handle_login_auto_create_account_is_disabled() {
        let packet = RequestAuthLogin {
            username: "test".to_string(),
            password: "test".to_string(),
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool.clone(), r, w).await;
        let result = player_actor.ask(packet).await;
        let user = user::Model::find_some_by_username(&db_pool, "test")
            .await
            .unwrap();
        assert!(result.is_err());
        assert!(user.is_none());
    }
    #[tokio::test]
    async fn test_handle_login_wrong_password() {
        let packet = RequestAuthLogin {
            username: "test".to_string(),
            password: "test".to_string(),
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        user_factory(&db_pool, |mut u| {
            u.username = "admin".to_owned();
            u.password = "$argon2id$v=19$m=19456,t=2,p=1$OnSjOZTt6Or9MxtqrcrGhw$GAY7oGKMMAQbd6tvWB96IjA6yxvZy2PMD2MEpHbmWS0".to_owned();
            u
        })
            .await;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool.clone(), r, w).await;
        let result = player_actor.ask(packet).await;
        let user = user::Model::find_some_by_username(&db_pool, "test")
            .await
            .unwrap();
        assert!(result.is_err());
        assert!(user.is_none());
    }
}

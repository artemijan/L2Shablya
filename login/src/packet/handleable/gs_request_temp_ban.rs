use crate::gs_client::GameServerClient;
use entities::entities::user;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::RequestTempBan;
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};

impl Message<RequestTempBan> for GameServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestTempBan,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        //ignore error updating an account
        let user_model = user::Model::find_by_username(&self.db_pool, &msg.account).await?;
        let mut active_record: user::ActiveModel = user_model.into();
        active_record.ban_duration = ActiveValue::Set(Some(msg.ban_duration));
        active_record.ban_ip = ActiveValue::Set(Some(self.ip.to_string()));
        active_record.save(&self.db_pool).await?;
        info!("[Account banned] OK: {:?}", msg.account);
        self.lc
            .update_ip_ban_list(&self.ip.to_string(), msg.ban_duration);
        self.lc.remove_player(&msg.account);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::spawn_gs_client_actor;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;
    use l2_core::shared_packets::write::SendablePacketBuffer;

    #[tokio::test]
    async fn handle() {
        let acc = "admin".to_string();
        let packet = RequestTempBan {
            account: acc.clone(),
            ban_duration: 5000,
            ip: "127.0.0.1".to_string(),
            buffer: SendablePacketBuffer::empty(),
        };
        let db_pool = get_test_db().await;
        user_factory(&db_pool, |mut u| {
            u.username = acc.clone();
            u.access_level = 0;
            u
        })
        .await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let actor = spawn_gs_client_actor(lc, db_pool.clone(), r, w).await;
        let res = actor.ask(packet).await;
        let user_model = user::Model::find_by_username(&db_pool, &acc).await.unwrap();
        assert!(res.is_ok());
        assert_eq!(user_model.ban_duration, Some(5000));
        assert_eq!(user_model.ban_ip, Some("127.0.0.1".to_string()));
    }
}

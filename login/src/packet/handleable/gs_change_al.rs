use crate::gs_client::GameServerClient;
use entities::entities::user;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::ChangeAccessLevel;
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};

impl Message<ChangeAccessLevel> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ChangeAccessLevel,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //ignore error updating an account
        let user_model = user::Model::find_by_username(&self.db_pool, &msg.account).await?;
        let user_id = user_model.id;
        let mut active_model: user::ActiveModel = user_model.into();
        active_model.access_level = ActiveValue::Set(msg.level);
        active_model.save(&self.db_pool).await?;
        info!("[change access level] OK {user_id:?}");
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
    async fn change_al() {
        let packet = ChangeAccessLevel {
            buffer: SendablePacketBuffer::empty(),
            account: "admin".to_string(),
            level: 100,
        };
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
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
        let gs_actor = spawn_gs_client_actor(lc, db_pool.clone(), r, w).await;
        let res = gs_actor.ask(packet).await;
        let user_model = user::Model::find_by_username(&db_pool, &acc).await.unwrap();
        assert_eq!(user_model.access_level, 100);
        assert!(res.is_ok());
    }
}

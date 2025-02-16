use entities::entities::user;
use l2_core::{
    shared_packets::gs_2_ls::ChangeAccessLevel,
    traits::handlers::PacketHandler,
};
use crate::gs_thread::GSHandler;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for ChangeAccessLevel {
    type HandlerType = GSHandler;

    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let db_pool = gs.get_db_pool();
        //ignore error updating an account
        let user_model = user::Model::find_by_username(db_pool, &self.account).await?;
        let user_id = user_model.id;
        let mut active_model: user::ActiveModel = user_model.into();
        active_model.access_level = ActiveValue::Set(self.level);
        active_model.save(db_pool).await?;
        info!("[change access level] OK {user_id:?}");
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use tokio::io::split;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::login::LoginServer;
    use l2_core::traits::ServerConfig;
    use test_utils::utils::get_test_db;
    use crate::controller::LoginController;
    use super::*;
    #[tokio::test]
    async fn change_al() {
        let packet = ChangeAccessLevel{
            account: "admin".to_string(),
            level: 100,
        };
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        user_factory(&db_pool, |mut u|{
            u.username = acc.clone();
            u.access_level = 0;
            u
        }).await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        let user_model = user::Model::find_by_username(&db_pool, &acc).await.unwrap();
        assert_eq!(user_model.access_level, 100);
        assert!(res.is_ok());
    }
}
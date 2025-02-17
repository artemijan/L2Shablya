use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use entities::entities::user;
use l2_core::{shared_packets::gs_2_ls::RequestTempBan, traits::handlers::PacketHandler};
use sea_orm::{ActiveModelTrait, ActiveValue};
use tracing::{info, instrument};

#[async_trait]
impl HandleablePacket for RequestTempBan {
    type HandlerType = GSHandler;
    #[instrument(skip(self, gs))]
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let db_pool = gs.get_db_pool();
        //ignore error updating an account
        let user_model = user::Model::find_by_username(db_pool, &self.account).await?;
        let mut active_record: user::ActiveModel = user_model.into();
        active_record.ban_duration = ActiveValue::Set(Some(self.ban_duration));
        active_record.ban_ip = ActiveValue::Set(Some(self.ip.clone()));
        active_record.save(db_pool).await?;
        info!("[Account banned] OK: {:?}", self.account);
        let lc = gs.get_controller();
        lc.update_ip_ban_list(&self.ip, self.ban_duration);
        lc.remove_player(&self.account);
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
    async fn handle() {
        let acc = "admin".to_string();
        let packet = RequestTempBan {
            account: acc.clone(),
            ban_duration: 5000,
            ip: "127.0.0.1".to_string(),
        };
        let db_pool = get_test_db().await;
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
        assert!(res.is_ok());
        assert_eq!(user_model.ban_duration, Some(5000));
        assert_eq!(user_model.ban_ip, Some("127.0.0.1".to_string()));
    }
}

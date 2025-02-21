use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::traits::Shutdown;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Logout;

impl ReadablePacket for Logout {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for Logout {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo handle proper logout mechanism: olympiad,
        // in battle state, on RB and so on, offline trade, etc...

        // evaluate user before using macro
        let user = handler.try_get_user()?;
        // there is a bug when ? doesn't propagate error inside macro
        info!("Player logged out: {user:}");
        handler.get_shutdown_listener().notify_one();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::gs::GSServer;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncWriteExt};

    #[tokio::test]
    async fn test() {
        let pool = get_test_db().await;
        let packet = Logout {};
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_user(user);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
        client.shutdown().await.unwrap();
    }
}

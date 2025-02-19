use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::gs_2_ls::{GSStatusUpdate, PlayerInGame};
use l2_core::shared_packets::ls_2_gs;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use std::sync::Arc;
use tracing::{info, instrument};

#[async_trait]
impl HandleablePacket for ls_2_gs::AuthGS {
    type HandlerType = LoginHandler;

    #[instrument(skip_all)]
    async fn handle(&self, lh: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = lh.get_controller();
        let cfg = controller.get_cfg();
        if self.server_id != cfg.server_id && !cfg.accept_alternative_id {
            bail!(
                "Can not accept alternative id from login server. Id is {}",
                self.server_id
            );
        }
        let gsu = GSStatusUpdate::new(&cfg)?;
        lh.send_packet(Box::new(gsu)).await?;
        info!(
            "Registered on Login server: {:} ({:})",
            self.server_name, self.server_id
        );
        controller
            .message_broker
            .register_packet_handler(Self::HandlerType::HANDLER_ID, Arc::new(lh.clone()));
        let accounts = controller.get_online_accounts();
        if !accounts.is_empty() {
            lh.send_packet(Box::new(PlayerInGame::new(&accounts)?))
                .await?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use l2_core::config::gs::GSServer;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use std::net::Ipv4Addr;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    #[tokio::test]
    #[timeout(3000)]
    async fn test_handle() {
        let pool = get_test_db().await;
        let pack = ls_2_gs::AuthGS::new(1, String::from("server"));
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = LoginHandler::new(r, w, Ipv4Addr::LOCALHOST, pool, controller);
        pack.handle(&mut ch).await.unwrap();
        tokio::spawn(async move {
            ch.handle_client().await.unwrap();
        });
        let mut resp = [0; 58];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(
            resp,
            [
                58, 0, 76, 251, 222, 141, 212, 88, 85, 240, 59, 90, 236, 153, 102, 10, 246, 176,
                250, 50, 72, 40, 202, 175, 76, 94, 24, 152, 9, 124, 31, 95, 243, 70, 220, 144, 201,
                170, 66, 227, 200, 114, 156, 170, 34, 76, 99, 155, 22, 190, 244, 25, 47, 134, 132,
                153, 79, 213
            ]
        );
        let mut resp_online_accs = [0; 26];
        client.read_exact(&mut resp_online_accs).await.unwrap();
        assert_eq!(
            [
                26, 0, 8, 98, 48, 179, 96, 39, 86, 199, 128, 50, 237, 5, 221, 64, 108, 178, 28,
                101, 240, 217, 254, 24, 199, 245
            ],
            resp_online_accs
        );
        client.shutdown().await.unwrap();
    }
}

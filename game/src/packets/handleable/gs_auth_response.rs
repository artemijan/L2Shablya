use crate::ls_client::LoginServerClient;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::gs_2_ls::{GSStatusUpdate, PlayerInGame};
use l2_core::shared_packets::ls_2_gs;
use tracing::{error, info, instrument};
use l2_core::traits::ServerToServer;

impl Message<ls_2_gs::AuthGS> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ls_2_gs::AuthGS,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let cfg = self.controller.get_cfg();
        if msg.server_id != cfg.server_id && !cfg.accept_alternative_id {
            error!(
                "Can not accept alternative id from login server. Id is {}",
                msg.server_id
            );
            return Ok(())
        }
        let gsu = GSStatusUpdate::new(&cfg)?;
        self.send_packet(gsu).await?;
        info!(
            "Registered on Login server: {:} ({:})",
            msg.server_name, msg.server_id
        );
        let accounts = self.controller.get_online_accounts();
        if !accounts.is_empty() {
            self.send_packet(PlayerInGame::new(&accounts)?)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::test_utils::test::spawn_ls_client_actor;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    async fn test_handle() {
        let pool = get_test_db().await;
        let pack = ls_2_gs::AuthGS::new(1, String::from("server"));
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg));
        controller.add_online_account("test", None);
        let actor = spawn_ls_client_actor(controller, pool, r, w).await;
        let res = actor.ask(pack).await;
        assert!(res.is_ok());
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
    }
}

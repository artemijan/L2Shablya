#[cfg(test)]
pub mod test {
    use crate::controller::Controller;
    use crate::ls_client::LoginServerClient;
    use entities::DBPool;
    use kameo::actor::ActorRef;
    use kameo::Actor;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use tokio::io::{DuplexStream, ReadHalf, WriteHalf};
    use crate::pl_client::PlayerClient;

    pub struct GetState;
    pub fn get_gs_config() -> GSServerConfig {
        GSServerConfig::from_string(include_str!("../../config/game.yaml"))
    }

    pub async fn spawn_custom_ls_client_actor(
        lc: Arc<Controller>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
        gs_client: Option<LoginServerClient>,
    ) -> ActorRef<LoginServerClient> {
        let ip = Ipv4Addr::LOCALHOST;
        let gs;
        if let Some(client) = gs_client {
            gs = client;
        } else {
            gs = LoginServerClient::new(ip, lc, db);
        }
        let gs_actor = LoginServerClient::spawn((gs, Box::new(r), Box::new(w)));
        gs_actor.wait_for_startup().await;
        gs_actor
    }
    pub async fn spawn_custom_player_client_actor(
        lc: Arc<Controller>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
        gs_client: Option<PlayerClient>,
    ) -> ActorRef<PlayerClient> {
        let ip = Ipv4Addr::LOCALHOST;
        let gs;
        if let Some(client) = gs_client {
            gs = client;
        } else {
            gs = PlayerClient::new(ip, lc, db);
        }
        let gs_actor = PlayerClient::spawn((gs, Box::new(r), Box::new(w)));
        gs_actor.wait_for_startup().await;
        gs_actor
    }
    pub async fn spawn_ls_client_actor(
        lc: Arc<Controller>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<LoginServerClient> {
        spawn_custom_ls_client_actor(lc, db, r, w, None).await
    }
    pub async fn spawn_player_client_actor(
        lc: Arc<Controller>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<PlayerClient> {
        spawn_custom_player_client_actor(lc, db, r, w, None).await
    }
}

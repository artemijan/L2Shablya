#[cfg(test)]
pub mod test {
    use crate::controller::GameController;
    use crate::ls_client::LoginServerClient;
    use crate::pl_client::PlayerClient;
    use entities::DBPool;
    use kameo::actor::{ActorRef, Spawn};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::path::PathBuf;
    use std::sync::{Arc, OnceLock};
    use tokio::io::{DuplexStream, ReadHalf, WriteHalf};
    static CONFIG: OnceLock<GSServerConfig> = OnceLock::new();
    pub fn get_gs_config() -> GSServerConfig {
        CONFIG
            .get_or_init(|| {
                // Use CARGO_MANIFEST_DIR to find the workspace root safely
                let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                path.pop();
                path.push("config");
                path.push("game.yaml");
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Could not find config at {}", path.display()));
                GSServerConfig::from_string(&content)
            })
            .clone()
    }

    pub async fn spawn_custom_ls_client_actor(
        lc: Arc<GameController>,
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
        lc: Arc<GameController>,
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
        lc: Arc<GameController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<LoginServerClient> {
        spawn_custom_ls_client_actor(lc, db, r, w, None).await
    }
    pub async fn spawn_player_client_actor(
        lc: Arc<GameController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<PlayerClient> {
        spawn_custom_player_client_actor(lc, db, r, w, None).await
    }
}

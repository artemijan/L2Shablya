#[cfg(test)]
pub mod test {
    use crate::controller::LoginController;
    use crate::gs_client::GameServerClient;
    use crate::login_client::LoginClient;
    use entities::DBPool;
    use kameo::actor::ActorRef;
    use kameo::message::{Context, Message};
    use kameo::Actor;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use tokio::io::{DuplexStream, ReadHalf, WriteHalf};

    pub struct GetState;

    impl Message<GetState> for LoginClient {
        type Reply = Arc<Self>;
        async fn handle(
            &mut self,
            msg: GetState,
            _ctx: &mut Context<Self, Self::Reply>,
        ) -> Self::Reply {
            Arc::new(self.clone())
        }
    }

    pub async fn spawn_custom_login_client_actor(
        lc: Arc<LoginController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
        login_client: Option<LoginClient>,
    ) -> ActorRef<LoginClient> {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let player;
        if let Some(client) = login_client {
            player = client;
        } else {
            player = LoginClient::new(ip, lc, db);
        }
        let player_actor = LoginClient::spawn((player, Box::new(r), Box::new(w)));
        player_actor.wait_for_startup().await;
        player_actor
    }
    pub async fn spawn_login_client_actor(
        lc: Arc<LoginController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<LoginClient> {
        spawn_custom_login_client_actor(lc, db, r, w, None).await
    }

    pub async fn spawn_custom_gs_client_actor(
        lc: Arc<LoginController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
        gs_client: Option<GameServerClient>,
    ) -> ActorRef<GameServerClient> {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let gs;
        if let Some(client) = gs_client {
            gs = client;
        } else {
            gs = GameServerClient::new(ip, lc, db);
        }
        let gs_actor = GameServerClient::spawn((gs, Box::new(r), Box::new(w)));
        gs_actor.wait_for_startup().await;
        gs_actor
    }
    pub async fn spawn_gs_client_actor(
        lc: Arc<LoginController>,
        db: DBPool,
        r: ReadHalf<DuplexStream>,
        w: WriteHalf<DuplexStream>,
    ) -> ActorRef<GameServerClient> {
        spawn_custom_gs_client_actor(lc, db, r, w, None).await
    }
}

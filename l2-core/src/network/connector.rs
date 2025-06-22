use crate::dto::OutboundConnection;
use kameo::actor::ActorRef;
use kameo::Actor;
use std::net::{IpAddr, Ipv4Addr};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tracing::{info, instrument};

pub struct Connector {
    pub name: String,
    pub cfg: OutboundConnection,
}

impl Connector {
    /// # Errors
    /// - when can't bind ip address and port
    ///
    /// # Panics
    /// - when Ipv6 connection
    pub fn run<F, Fut, A>(&self, on_connect: F) -> JoinHandle<()>
    where
        F: FnOnce(TcpStream, Ipv4Addr) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ActorRef<A>> + Send + 'static,
        A: Actor,
    {
        info!(
            "[{}] Listening on {}:{}",
            self.name, self.cfg.ip, self.cfg.port
        );
        let cfg = self.cfg.clone();
        let handler_name = self.name.clone();
        tokio::spawn(async move {
            let address = format!("{}:{}", cfg.ip, cfg.port);
            info!("Connecting to {} on: {}", handler_name, &address);
            loop {
                let stream = Self::get_stream(&address).await;
                stream
                    .set_nodelay(cfg.no_delay)
                    .expect("Set nodelay failed");
                let IpAddr::V4(ip) = stream.peer_addr().expect("Cannot get peer address").ip()
                else {
                    tracing::error!("IP v6 is not supported");
                    break;
                };
                let handler = on_connect.clone();
                let actor = handler(stream, ip).await;
                actor.wait_for_shutdown().await;
                info!("{handler_name}: Lost connection to {ip}, trying again in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        })
    }
    #[instrument]
    async fn get_stream(address: &str) -> TcpStream {
        loop {
            match TcpStream::connect(address).await {
                Ok(s) => break s, // Exit the loop when the connection succeeds
                Err(e) => {
                    tracing::error!(
                        "Failed to connect to {address}: {e}. Retrying in 5 seconds..."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

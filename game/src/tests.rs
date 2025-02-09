use std::sync::Arc;
use l2_core::config::gs::GSServer;
use l2_core::traits::ServerConfig;
use tokio::sync::Mutex;
use tokio::io::AsyncWrite;
use std::fmt;
use async_trait::async_trait;
use l2_core::traits::handlers::PacketSender;

pub fn get_gs_config() ->Arc<GSServer>{
    Arc::new(GSServer::from_string(include_str!(
        "../test_data/game.yaml"
    )))
}

pub struct TestPacketSender {
    pub writer: Arc<Mutex<dyn AsyncWrite + Unpin + Send>>,
}

impl fmt::Debug for TestPacketSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestPacketSender")
    }
}

#[async_trait]
impl PacketSender for TestPacketSender {
    async fn encrypt(&self, _: &mut [u8]) -> anyhow::Result<()> {
        Ok(())
    }

    fn is_encryption_enabled(&self) -> bool {
        false
    }

    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<dyn AsyncWrite + Send + Unpin>> {
        &self.writer
    }
}
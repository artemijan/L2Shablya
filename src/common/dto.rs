use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    #[serde(rename = "url")]
    pub url: String,
    pub max_connections: u8,
    pub min_connections: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Runtime {
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct InboundConnection {
    pub ip: String,
    pub port: u16,
    pub reuse_addr: bool,
    pub reuse_port: bool,
    pub no_delay: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OutboundConnection {
    pub ip: String,
    pub port: u16,
    pub no_delay: bool,
}

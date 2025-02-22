use pnet::ipnetwork::Ipv4Network;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Database {
    #[serde(rename = "url")]
    pub url: String,
    pub max_connections: u8,
    pub min_connections: u8,
    #[serde(default = "default_timeout")]
    pub connect_timeout: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
}
#[derive(Clone, Debug)]
pub struct Player {
    pub login_name: String,
}
fn default_idle_timeout() -> u64 {
    60
}
fn default_timeout() -> u64 {
    10
}
fn default_max_lifetime() -> u64 {
    60 * 60
}
#[derive(Debug, Clone, Deserialize, Default)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ServerHost {
    #[serde(deserialize_with = "deserialize_ip")]
    pub ip: Ipv4Addr,
    #[serde(deserialize_with = "deserialize_subnet")]
    pub subnet: Ipv4Network,
}

fn deserialize_ip<'de, D>(deserializer: D) -> Result<Ipv4Addr, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ipv4Addr::from_str(&s).map_err(Error::custom)
}

fn deserialize_subnet<'de, D>(deserializer: D) -> Result<Ipv4Network, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ipv4Network::from_str(&s).map_err(Error::custom)
}

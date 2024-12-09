use crate::common::dto::{Database, InboundConnection, OutboundConnection, Runtime, ServerHost};
use crate::common::packets::common::ServerType;
use crate::common::traits::ServerConfig;
use log::{error, info};
use num::BigInt;
use num_traits::Num;
use pnet::datalink;
use reqwest::blocking;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
pub struct GSServer {
    pub name: String,
    pub blowfish_key: String,
    pub runtime: Option<Runtime>,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
    #[serde(deserialize_with = "deserialize_hex_to_bigint")]
    pub hex_id: BigInt,
    pub server_id: u8,
    pub accept_alternative_id: bool,
    pub host_reserved: bool,
    pub use_brackets: bool,
    pub max_players: u32,
    #[serde(deserialize_with = "deserialize_server_type")]
    pub server_type: ServerType,
    pub server_age: u8,
    pub gm_only: bool,
    #[serde(default)]
    pub ip_config: Vec<ServerHost>,
}
impl GSServer {
    pub fn get_hosts(&self) -> Vec<String> {
        self.ip_config
            .iter()
            .flat_map(|h| vec![h.subnet.to_string(), h.ip.to_string()])
            .collect()
    }
}

fn deserialize_hex_to_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    BigInt::from_str_radix(&s, 16).map_err(Error::custom)
}
fn deserialize_server_type<'de, D>(deserializer: D) -> Result<ServerType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ServerType::from_str(&s).map_err(Error::custom)
}

impl GSServer {
    fn auto_ip_config(&mut self) {
        // Get all network interfaces
        let interfaces = datalink::interfaces();
        // Filter and collect pairs of (subnet, IP address)
        for i_face in &interfaces {
            for ip in &i_face.ips {
                if let pnet::ipnetwork::IpNetwork::V4(ipv4) = ip {
                    self.ip_config.push(ServerHost {
                        ip: ipv4.ip(),
                        subnet: *ipv4,
                    });
                }
            }
        }
        let Ok(resp) = blocking::get("https://checkip.amazonaws.com/") else {
            return;
        };
        if !resp.status().is_success() {
            return;
        }
        let Ok(external_ip) = resp.text() else {
            return;
        };
        if let Ok(ip) = Ipv4Addr::from_str(external_ip.trim()) {
            self.ip_config.push(ServerHost {
                ip,
                subnet: "0.0.0.0/0".parse().unwrap(),
            });
        } else {
            error!("Failed to parse external IP address: {}", external_ip);
        }
    }
}
impl ServerConfig for GSServer {
    fn load(file_name: &str) -> Self {
        let file = File::open(file_name)
            .unwrap_or_else(|e| panic!("Failed to open config file: {file_name}. Error: {e}"));
        let reader = BufReader::new(file);
        let mut config: GSServer = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!("Unable to parse {file_name}, the format is incorrect, {e}")
        });
        println!("Configuration ok, starting application: {}", config.name);
        if config.ip_config.is_empty() {
            info!("Missing ip config, using autoconfiguration");
            config.auto_ip_config();
        }
        config
    }

    fn from_string(conf: &str) -> Self {
        serde_yaml::from_str::<GSServer>(conf)
            .unwrap_or_else(|e| panic!("Unable to parse {conf}, the format is incorrect, {e}"))
    }

    fn runtime(&self) -> Option<&Runtime> {
        self.runtime.as_ref()
    }

    fn database(&self) -> &Database {
        &self.database
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientListener {
    pub connection: InboundConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginServerConnector {
    pub connection: OutboundConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listeners {
    pub clients: ClientListener,
    pub login_server: LoginServerConnector,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub timeout: u8,
}

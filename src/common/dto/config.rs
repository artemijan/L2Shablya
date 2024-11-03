use num::{BigInt, Num};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub name: String,
    pub blowfish_key: String,
    pub runtime: Option<Runtime>,
    pub auto_registration: bool,
    #[serde(deserialize_with = "validate_allowed_gs_keys")]
    pub allowed_gs: Option<HashMap<String, AllowedGS>>,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
}

impl Server {
    pub fn load(file_name: &str) -> Self {
        let file = File::open(file_name)
            .unwrap_or_else(|e| panic!("Failed to open config file: {file_name}. Error: {e}"));
        let reader = BufReader::new(file);
        let config: Server = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!("Unable to parse {file_name}, the format is incorrect, {e}")
        });
        println!("Configuration ok, starting application: {}", config.name);
        config
    }
}

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

// Custom deserialization function to validate that all keys in the HashMap are valid hex strings
fn validate_allowed_gs_keys<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, AllowedGS>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the HashMap first
    let map: Option<HashMap<String, AllowedGS>> = Option::deserialize(deserializer)?;

    if let Some(map) = &map {
        for key in map.keys() {
            // Check if each key is a valid hex string and convertible to a BigInt
            if BigInt::from_str_radix(key, 16).is_err() {
                return Err(Error::custom(format!("Invalid hex key: '{key}'")));
            }
        }
    }
    Ok(map)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connection {
    pub ip: String,
    pub port: u16,
    pub reuse_addr: bool,
    pub reuse_port: bool,
    pub no_delay: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GSMessages {
    pub timeout: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GSListener {
    pub connection: Connection,
    pub messages: GSMessages,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ClientListener {
    pub connection: Connection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowedGS {
    server_id: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listeners {
    pub game_servers: GSListener,
    pub clients: ClientListener,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub timeout: u8,
    pub show_licence: bool,
    pub enable_cmdline_login: bool,
}

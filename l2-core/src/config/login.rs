use crate::dto::{Database, InboundConnection, Runtime};
use crate::traits::ServerConfig;
use num::BigInt;
use num_traits::Num;
use serde::{Deserialize, Deserializer};
use std::env;
use std::fs::File;
use std::io::BufReader;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginServer {
    pub name: String,
    pub blowfish_key: String,
    pub runtime: Option<Runtime>,
    #[serde(deserialize_with = "validate_allowed_gs_keys")]
    pub allowed_gs: Option<Vec<BigInt>>,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
}

impl ServerConfig for LoginServer {
    fn load(file_name: &str) -> Self {
        let file = File::open(file_name).unwrap_or_else(|e| {
            let cwd = env::current_dir()
                .map_or_else(|_| "unknown".to_string(), |path| path.display().to_string());
            panic!("Failed to open config file: {file_name}. Error: {e}. Current directory: {cwd}");
        });
        let reader = BufReader::new(file);
        let config: LoginServer = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!("Unable to parse {file_name}, the format is incorrect, {e}")
        });
        info!("Configuration ok, starting application: {}", config.name);
        config
    }
    fn from_string(conf: &str) -> Self {
        serde_yaml::from_str::<LoginServer>(conf)
            .unwrap_or_else(|e| panic!("Unable to parse {conf}, the format is incorrect, {e}"))
    }

    fn runtime(&self) -> Option<&Runtime> {
        self.runtime.as_ref()
    }
    fn database(&self) -> &Database {
        &self.database
    }
}

// Custom deserialization function to validate that all keys in the HashMap are valid hex strings
fn validate_allowed_gs_keys<'de, D>(deserializer: D) -> Result<Option<Vec<BigInt>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the list of strings
    let list: Option<Vec<String>> = Option::deserialize(deserializer)?;

    if let Some(severs) = list {
        // Try to convert the keys into BigInt, returning an error if any are invalid
        let mut converted = Vec::new();
        for key in severs {
            match BigInt::from_str_radix(&key, 16) {
                Ok(big_int) => converted.push(big_int),
                Err(_) => {
                    return Err(serde::de::Error::custom(format!(
                        "Invalid hex key: '{key}'"
                    )));
                }
            }
        }
        return Ok(Some(converted));
    }

    Ok(None)
}

#[derive(Debug, Clone, Deserialize)]
pub struct GSListener {
    pub connection: InboundConnection,
    pub messages: GSMessages,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientListener {
    pub connection: InboundConnection,
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
    pub auto_create_accounts: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GSMessages {
    pub timeout: u8,
}

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{env, fs};

const CONFIG_PATH_ENV: &str = "CONFIG_PATH";
const CONFIG_DEFAULT_PATH: &str = "config/config.yaml";

pub fn load_config() -> Result<Config, crate::Error> {
    let config_path = env::var(CONFIG_PATH_ENV);
    let raw_config = match config_path {
        Ok(path) => fs::read_to_string(path)?,
        _ => fs::read_to_string(CONFIG_DEFAULT_PATH)?,
    };

    let config: Config = serde_yaml::from_str(&raw_config)?;
    Ok(config)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub chain: ChainConfig,
    pub node: NodeConfig,
    pub parsing: ParserConfig,
    pub database: DBConfig,
    pub logging: LogginConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub modules: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub config: NodeDetail,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeDetail {
    pub rpc: RPCConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCConfig {
    pub client_name: String,
    pub address: String,
    pub max_connections: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParserConfig {
    pub workers: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_file_path: Option<String>,
    pub start_height: u64,
    pub listen_new_blocks: bool,
    pub parse_old_blocks: bool,
    pub parse_genesis: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBConfig {
    pub url: String,
    pub max_open_connections: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogginConfig {
    pub level: String,
    pub format: String,
}

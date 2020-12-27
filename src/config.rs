use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: Vec<ServerConfig>,
    pub plugin_locations: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub ssl: bool,
    pub nicknames: Option<Vec<String>>,
    pub plugin_whitelist: Option<Vec<String>>,
    pub plugin_blacklist: Option<Vec<String>>,
    pub channel: Option<Vec<ChannelConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelConfig {
    pub name: String,
    pub key: Option<String>,
    pub plugin_whitelist: Option<Vec<String>>,
    pub plugin_blacklist: Option<Vec<String>>,
}

fn get_or_create_config_file() -> Result<String, std::io::Error> {
    let app_dirs = AppDirs::new(Some("neatbot"), true).unwrap();
    let config_path = app_dirs.config_dir.join("neatbot.toml");

    fs::create_dir_all(&app_dirs.config_dir)?;

    if !config_path.exists() {
        File::create(&config_path).unwrap();
    };

    fs::read_to_string(&config_path)
}

impl Config {
    pub fn from_config_folder() -> Result<Config> {
        let config_contents = get_or_create_config_file()?;
        if config_contents.is_empty() {
            return Err(anyhow!("empty config file"));
        }

        let parsed_config: Config = toml::from_str(&config_contents)?;

        Ok(parsed_config)
    }
}

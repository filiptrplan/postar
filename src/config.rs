use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub imap: Vec<IMAPConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IMAPConfig {
    pub name: String,
    pub server: String,
    pub port: u16,
    #[serde(default = "default_self_signed_cert")]
    pub self_signed_cert: bool,
    pub username: String,
    pub password: String,
    #[serde(default = "default_default")]
    pub default: bool,
}

fn default_self_signed_cert() -> bool {
    false
}

fn default_default() -> bool {
    false
}

impl Config {
    /// Load from config from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let buf = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&buf)?;
        Ok(cfg)
    }
}

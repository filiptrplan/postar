use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub imap: Vec<IMAPConfig>,
    #[serde(default)]
    pub postar: PostarConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PostarConfig {
    pub polling_delay: u32,
}

impl Default for PostarConfig {
    fn default() -> Self {
        Self { polling_delay: 3 }
    }
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
    #[serde(default = "default_incoming_folder")]
    pub incoming_folder: String,
}

fn default_incoming_folder() -> String {
    String::from("INBOX")
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

    /// Merge with the main CLI args struct
    pub fn merge_with_args(mut self, args: &crate::cli::args::Args) -> Self {
        self.postar.polling_delay = args.polling_delay.unwrap_or(self.postar.polling_delay);
        self
    }
}

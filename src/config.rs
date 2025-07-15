use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8000
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub workspaces: std::collections::HashMap<String, Workspace>,
    #[serde(default)]
    pub server: ServerConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: default_port(),
        }
    }
}

impl Config {
    pub fn load_config() -> Result<Self> {
        let config_path = get_config_path();

        if !config_path.exists() {
            return Err(anyhow!("config file not found at {:?}", config_path));
        }

        let data = fs::read_to_string(&config_path)?;
        let cfg: Config = serde_yaml::from_str(&data)?;

        Ok(cfg)
    }

    pub fn get_path(&self, workspace_key: &str) -> Result<&PathBuf> {
        let key = if workspace_key.is_empty() {
            self.workspaces
                .keys()
                .next()
                .ok_or_else(|| anyhow!("no workspaces defined in config"))?
        } else {
            workspace_key
        };

        let workspace = self
            .workspaces
            .get(key)
            .ok_or_else(|| anyhow!("workspace '{}' not found in config", key))?;

        if !workspace.path.exists() {
            return Err(anyhow!(
                "workspace '{}' does not exist: {:?}",
                key,
                workspace.path
            ));
        }

        Ok(&workspace.path)
    }
}

fn get_config_path() -> PathBuf {
    if cfg!(windows) {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\"))
            .join("AppData")
            .join("Roaming")
            .join("arrow")
            .join("arrow.conf")
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/etc"))
            .join("arrow")
            .join("arrow.conf")
    }
}

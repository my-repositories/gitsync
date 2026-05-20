use anyhow::{Context, Result};
use std::{env, path::PathBuf};
use tokio::fs;

use crate::configuration::config_settings::ConfigSettings;

pub struct ConfigReader;

impl ConfigReader {
    pub async fn read_config() -> Result<ConfigSettings> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Config file not found: {}",
                config_path.display()
            ));
        }

        let json = fs::read_to_string(&config_path)
            .await
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let cfg: ConfigSettings =
            serde_json::from_str(&json).context("Failed to deserialize config")?;

        Ok(cfg)
    }

    fn get_config_path() -> PathBuf {
        if let Ok(path) = env::var("GITSYNC_CONFIG_PATH") {
            if !path.trim().is_empty() {
                return PathBuf::from(path);
            }
        }

        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".gitsync").join("config.json")
    }
}
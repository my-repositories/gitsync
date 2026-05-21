use anyhow::{Context, Result};
use std::{env, path::PathBuf};
use tokio::fs;

use crate::configuration::config_settings::ConfigSettings;

pub struct ConfigReader;

impl ConfigReader {
    pub async fn read_config() -> Result<ConfigSettings> {
        let config_path = Self::get_config_path();
        Self::read_config_from_path(config_path).await
    }

    pub async fn read_config_from_path(config_path: PathBuf) -> Result<ConfigSettings> {
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

    pub fn get_config_path() -> PathBuf {
        let config_env = env::var("GITSYNC_CONFIG_PATH").ok();
        let home_env = dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string()));
        
        Self::resolve_config_path(config_env.as_deref(), home_env.as_deref())
    }

    pub fn resolve_config_path(config_path: Option<&str>, home: Option<&str>) -> PathBuf {
        if let Some(path) = config_path.map(str::trim).filter(|s| !s.is_empty()) {
            return PathBuf::from(path);
        }

        let home = home.unwrap_or(".");
        PathBuf::from(home).join(".gitsync").join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_from_env() {
        let path =
            ConfigReader::resolve_config_path(Some("/tmp/my-config.json"), Some("/home/tester"));
        assert_eq!(path, PathBuf::from("/tmp/my-config.json"));
    }

    #[test]
    fn config_path_from_home() {
        let path = ConfigReader::resolve_config_path(None, Some("/home/tester"));
        assert_eq!(path, PathBuf::from("/home/tester/.gitsync/config.json"));
    }

    #[test]
    fn config_path_fallback_to_dot() {
        let path = ConfigReader::resolve_config_path(None, None);
        assert_eq!(path, PathBuf::from("./.gitsync/config.json"));
    }

    #[test]
    fn config_path_ignores_empty_env() {
        let path = ConfigReader::resolve_config_path(Some("   "), Some("/home/tester"));
        assert_eq!(path, PathBuf::from("/home/tester/.gitsync/config.json"));
    }
}

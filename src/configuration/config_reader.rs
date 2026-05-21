use anyhow::{Context, Result};
use std::{env, path::PathBuf};
use tokio::fs;

use crate::configuration::config_settings::ConfigSettings;

pub struct ConfigReader;

impl ConfigReader {
    pub async fn read_config() -> Result<ConfigSettings> {
        let config_paths = Self::collect_config_paths();

        if config_paths.is_empty() {
            return Err(anyhow::anyhow!(
                "Config file not found in any of the expected locations."
            ));
        }

        let mut final_cfg: Option<ConfigSettings> = None;

        for path in config_paths {
            let cfg = Self::read_config_from_path(path).await?;
            if let Some(ref mut current_base) = final_cfg {
                current_base.merge(cfg);
            } else {
                final_cfg = Some(cfg);
            }
        }

        final_cfg.context("Failed to build configuration")
    }

    pub async fn read_config_from_path(config_path: PathBuf) -> Result<ConfigSettings> {
        let json = fs::read_to_string(&config_path)
            .await
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let cfg: ConfigSettings =
            serde_json::from_str(&json).context("Failed to deserialize config")?;

        Ok(cfg)
    }

    pub fn collect_config_paths() -> Vec<PathBuf> {
        Self::resolve_config_hierarchy(
            env::var("GITSYNC_CONFIG_PATH").ok(),
            dirs::home_dir(),
            env::current_dir().ok(),
        )
    }

    pub fn resolve_config_hierarchy(
        config_env: Option<String>,
        home_dir: Option<PathBuf>,
        current_dir: Option<PathBuf>,
    ) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. ENV переменная (Абсолютный приоритет)
        if let Some(env_path) = config_env
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| p.exists())
        {
            paths.push(env_path);
            return paths;
        }

        // 2. Глобальный конфиг (~/.gitsync/config.json)
        if let Some(home) = home_dir {
            let global_path = home.join(".gitsync").join("config.json");
            if global_path.exists() {
                paths.push(global_path);
            }
        }

        if let Some(current) = current_dir {
            // 3. subdivision конфиг (Родительская папка)
            if let Some(parent) = current.parent() {
                let parent_path = parent.join(".gitsync").join("config.json");
                if parent_path.exists() && !paths.contains(&parent_path) {
                    paths.push(parent_path);
                }
            }

            // 4. Локальный конфиг (Текущая папка проекта)
            let local_path = current.join(".gitsync").join("config.json");
            if local_path.exists() && !paths.contains(&local_path) {
                paths.push(local_path);
            }
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir_all};

    #[test]
    fn test_env_takes_absolute_priority() {
        let temp = tempfile::tempdir().unwrap();
        let config_file = temp.path().join("custom-config.json");
        File::create(&config_file).unwrap();

        let paths = ConfigReader::resolve_config_hierarchy(
            Some(config_file.to_str().unwrap().to_string()),
            Some(PathBuf::from("/home/user")),
            Some(PathBuf::from("/home/user/repo")),
        );

        assert_eq!(paths.len(), 1);
        assert_eq!(paths, vec![config_file]);
    }

    #[test]
    fn test_hierarchy_loading_order() {
        let temp = tempfile::tempdir().unwrap();

        // Эмулируем структуру папок: /tmp/fake_home/org_dir/project_dir
        let fake_home = temp.path().join("fake_home");
        let org_dir = fake_home.join("org_dir");
        let project_dir = org_dir.join("project_dir");

        let global_config = fake_home.join(".gitsync").join("config.json");
        let org_config = org_dir.join(".gitsync").join("config.json");
        let local_config = project_dir.join(".gitsync").join("config.json");

        create_dir_all(global_config.parent().unwrap()).unwrap();
        create_dir_all(org_config.parent().unwrap()).unwrap();
        create_dir_all(local_config.parent().unwrap()).unwrap();

        File::create(&global_config).unwrap();
        File::create(&org_config).unwrap();
        File::create(&local_config).unwrap();

        // Запускаем поиск по нашей фейковой иерархии
        let paths =
            ConfigReader::resolve_config_hierarchy(None, Some(fake_home), Some(project_dir));

        // Должно найти строго 3 конфига в правильном порядке наслоения
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], global_config); // 1. Базовый
        assert_eq!(paths[1], org_config); // 2. Организация
        assert_eq!(paths[2], local_config); // 3. Проект (самый локальный)
    }

    #[test]
    fn test_empty_hierarchy_returns_nothing() {
        let paths = ConfigReader::resolve_config_hierarchy(
            None,
            Some(PathBuf::from("/nonexistent/home")),
            Some(PathBuf::from("/nonexistent/repo")),
        );
        assert!(paths.is_empty());
    }
}

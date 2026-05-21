use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub log_level: String,
    pub remote_branch_template: String,
    pub remote_urls: HashMap<String, String>,
    pub source_remote_name: String,
}

impl ConfigSettings {
    pub fn merge(&mut self, local: ConfigSettings) {
        if !local.log_level.trim().is_empty() {
            self.log_level = local.log_level;
        }
        
        if !local.remote_branch_template.trim().is_empty() {
            self.remote_branch_template = local.remote_branch_template;
        }
        
        if !local.source_remote_name.trim().is_empty() {
            self.source_remote_name = local.source_remote_name;
        }

        self.remote_urls.extend(local.remote_urls);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_pascal_case_config() {
        let json = r#"
        {
            "LogLevel": "Information",
            "RemoteBranchTemplate": "%owner%/%reponame%/%branchname%",
            "RemoteUrls": {
                "mirror1": "git@github.com:owner/repo.git",
                "mirror2": "git@gitlab.com:owner/repo.git"
            },
            "SourceRemoteName": "origin"
        }
        "#;

        let cfg: ConfigSettings = serde_json::from_str(json).unwrap();

        assert_eq!(cfg.log_level, "Information");
        assert_eq!(
            cfg.remote_branch_template,
            "%owner%/%reponame%/%branchname%"
        );
        assert_eq!(cfg.source_remote_name, "origin");
        assert_eq!(cfg.remote_urls.len(), 2);
        assert_eq!(cfg.remote_urls["mirror1"], "git@github.com:owner/repo.git");
    }

    #[test]
    fn test_config_merge_logic() {
        // 1. Создаем глобальный конфиг со стандартными настройками
        let mut global_cfg = ConfigSettings {
            log_level: "Information".to_string(),
            remote_branch_template: "global/template".to_string(),
            source_remote_name: "origin".to_string(),
            remote_urls: HashMap::from([
                ("global_mirror".to_string(), "git@github.com:global.git".to_string()),
                ("override_mirror".to_string(), "git@github.com:old.git".to_string()),
            ]),
        };

        // 2. Локальный конфиг (например, для конкретного проекта)
        let local_cfg = ConfigSettings {
            log_level: "Debug".to_string(), // меняем уровень логирования
            remote_branch_template: "".to_string(), // оставляем пустым, должно сохраниться глобальное
            source_remote_name: "upstream".to_string(), // меняем сорс
            remote_urls: HashMap::from([
                ("override_mirror".to_string(), "git@github.com:new.git".to_string()), // перебиваем юрл
                ("local_mirror".to_string(), "git@github.com:local.git".to_string()), // добавляем новое
            ]),
        };

        // Мержим
        global_cfg.merge(local_cfg);

        // 3. Проверяем результаты
        assert_eq!(global_cfg.log_level, "Debug"); // Перезаписалось
        assert_eq!(global_cfg.remote_branch_template, "global/template"); // Сохранилось, так как в локальном была пустота
        assert_eq!(global_cfg.source_remote_name, "upstream"); // Перезаписалось
        
        // Проверяем мапу юрлов
        assert_eq!(global_cfg.remote_urls.len(), 3); // Итого 3 ссылки
        assert_eq!(global_cfg.remote_urls["global_mirror"], "git@github.com:global.git"); // Осталось от глобального
        assert_eq!(global_cfg.remote_urls["override_mirror"], "git@github.com:new.git"); // Локальное затерло глобальное
        assert_eq!(global_cfg.remote_urls["local_mirror"], "git@github.com:local.git"); // Добавилось из локального
    }
}

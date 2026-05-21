use serde::Deserialize;
use std::collections::HashMap;

/// Финальная конфигурация для всего приложения (чистые типы, без Option)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub log_level: String,
    pub remote_branch_template: String,
    pub remote_urls: HashMap<String, String>,
    pub source_remote_name: String,
}

/// Промежуточная структура для частичной десериализации файлов и мержинга слоев
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PartialConfigSettings {
    pub log_level: Option<String>,
    pub remote_branch_template: Option<String>,
    pub remote_urls: Option<HashMap<String, String>>,
    pub source_remote_name: Option<String>,
}

impl PartialConfigSettings {
    /// Накладывает локальные заполненные настройки поверх текущих
    pub fn merge(&mut self, local: PartialConfigSettings) {
        if local.log_level.is_some() {
            self.log_level = local.log_level;
        }
        if local.remote_branch_template.is_some() {
            self.remote_branch_template = local.remote_branch_template;
        }
        if local.source_remote_name.is_some() {
            self.source_remote_name = local.source_remote_name;
        }
        if let Some(local_urls) = local.remote_urls {
            if let Some(ref mut base_urls) = self.remote_urls {
                base_urls.extend(local_urls);
            } else {
                self.remote_urls = Some(local_urls);
            }
        }
    }

    /// Преобразует опциональный конфиг в валидный ConfigSettings с дефолтными значениями,
    /// если какие-то критичные поля так и не были найдены в файлах.
    pub fn into_final_settings(self) -> ConfigSettings {
        ConfigSettings {
            log_level: self.log_level.unwrap_or_else(|| "Information".to_string()),
            remote_branch_template: self
                .remote_branch_template
                .unwrap_or_else(|| "%owner%/%reponame%/%branchname%".to_string()),
            remote_urls: self.remote_urls.unwrap_or_default(),
            source_remote_name: self
                .source_remote_name
                .unwrap_or_else(|| "origin".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_pascal_case_partial_config() {
        let json = r#"{"LogLevel": "Information"}"#;
        let cfg: PartialConfigSettings = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.log_level.as_deref(), Some("Information"));
        assert!(cfg.source_remote_name.is_none());
    }

    #[test]
    fn test_partial_merge_and_finalize() {
        let mut base = PartialConfigSettings {
            log_level: Some("Info".to_string()),
            source_remote_name: Some("origin".to_string()),
            ..Default::default()
        };

        let local = PartialConfigSettings {
            log_level: Some("Debug".to_string()),
            remote_urls: Some(HashMap::from([("m1".to_string(), "url".to_string())])),
            ..Default::default()
        };

        base.merge(local);
        let final_cfg = base.into_final_settings();

        assert_eq!(final_cfg.log_level, "Debug");
        assert_eq!(final_cfg.source_remote_name, "origin");
        assert_eq!(final_cfg.remote_urls["m1"], "url");
        assert_eq!(
            final_cfg.remote_branch_template,
            "%owner%/%reponame%/%branchname%"
        );
    }
}

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub log_level: Option<String>,
    pub remote_branch_template: Option<String>,
    pub remote_urls: Option<HashMap<String, String>>,
    pub source_remote_name: Option<String>,
}

impl ConfigSettings {
    /// Объединяет текущий конфиг (базовый) с более локальным.
    /// Локальные заполненные поля полностью заменяют или дополняют глобальные.
    pub fn merge(&mut self, local: ConfigSettings) {
        if local.log_level.is_some() {
            self.log_level = local.log_level;
        }

        if local.remote_branch_template.is_some() {
            self.remote_branch_template = local.remote_branch_template;
        }

        if local.source_remote_name.is_some() {
            self.source_remote_name = local.source_remote_name;
        }

        // Умный мерж HashMap
        if let Some(local_urls) = local.remote_urls {
            if let Some(ref mut base_urls) = self.remote_urls {
                base_urls.extend(local_urls);
            } else {
                self.remote_urls = Some(local_urls);
            }
        }
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

        assert_eq!(cfg.log_level.as_deref(), Some("Information"));
        assert_eq!(
            cfg.remote_branch_template.as_deref(),
            Some("%owner%/%reponame%/%branchname%")
        );
        assert_eq!(cfg.source_remote_name.as_deref(), Some("origin"));
        assert_eq!(cfg.remote_urls.as_ref().unwrap().len(), 2);
        assert_eq!(
            cfg.remote_urls.as_ref().unwrap()["mirror1"],
            "git@github.com:owner/repo.git"
        );
    }

    #[test]
    fn test_config_merge_logic() {
        let mut global_cfg = ConfigSettings {
            log_level: Some("Information".to_string()),
            remote_branch_template: Some("global/template".to_string()),
            source_remote_name: Some("origin".to_string()),
            remote_urls: Some(HashMap::from([
                (
                    "global_mirror".to_string(),
                    "git@github.com:global.git".to_string(),
                ),
                (
                    "override_mirror".to_string(),
                    "git@github.com:old.git".to_string(),
                ),
            ])),
        };

        let local_cfg = ConfigSettings {
            log_level: Some("Debug".to_string()),
            remote_branch_template: None, // В локальном конфиге поля может не быть вообще
            source_remote_name: Some("upstream".to_string()),
            remote_urls: Some(HashMap::from([
                (
                    "override_mirror".to_string(),
                    "git@github.com:new.git".to_string(),
                ),
                (
                    "local_mirror".to_string(),
                    "git@github.com:local.git".to_string(),
                ),
            ])),
        };

        global_cfg.merge(local_cfg);

        assert_eq!(global_cfg.log_level.as_deref(), Some("Debug"));
        assert_eq!(
            global_cfg.remote_branch_template.as_deref(),
            Some("global/template")
        );
        assert_eq!(global_cfg.source_remote_name.as_deref(), Some("upstream"));

        let urls = global_cfg.remote_urls.unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls["global_mirror"], "git@github.com:global.git");
        assert_eq!(urls["override_mirror"], "git@github.com:new.git");
        assert_eq!(urls["local_mirror"], "git@github.com:local.git");
    }
}

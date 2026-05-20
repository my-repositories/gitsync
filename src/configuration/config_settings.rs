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
}

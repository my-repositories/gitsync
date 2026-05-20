use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub log_level: String,
    pub remote_branch_template: String,
    pub remote_urls: HashMap<String, String>,
    pub source_remote_name: String,
}
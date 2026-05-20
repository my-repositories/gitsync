mod configuration;

use std::collections::HashMap;

use configuration::config_settings::ConfigSettings;

fn main() {
    let mut remote_urls = HashMap::new();
    remote_urls.insert(
        "gitlab".to_string(),
        "git@gitlab.com:loktionov129/gitsync.git".to_string(),
    );
    remote_urls.insert(
        "gitverse".to_string(),
        "git@gitverse.ru:loktionov129/gitsync.git".to_string(),
    );

    let cfg = ConfigSettings {
        log_level: "Debug".to_string(),
        source_remote_name: "origin".to_string(),
        remote_branch_template: "%owner%/%reponame%/%branchname%".to_string(),
        remote_urls: remote_urls,
    };

    println!("{:#?}", cfg);
}
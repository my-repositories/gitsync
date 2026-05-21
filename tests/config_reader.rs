use std::io::Write;
use tempfile::NamedTempFile;

use gitsync::configuration::config_reader::ConfigReader;

#[tokio::test]
async fn reads_config_from_temp_file() {
    let mut file = NamedTempFile::new().unwrap();

    write!(
        file,
        r#"{{
            "LogLevel": "info",
            "RemoteBranchTemplate": "%owner%/%reponame%/%branchname%",
            "RemoteUrls": {{
                "mirror1": "git@github.com:owner/repo.git",
                "mirror2": "git@gitlab.com:owner/repo.git"
            }},
            "SourceRemoteName": "origin"
        }}"#
    )
    .unwrap();

    // Получаем PartialConfigSettings
    let partial_cfg = ConfigReader::read_config_from_path(file.path().to_path_buf())
        .await
        .unwrap();

    // Превращаем в финальный чистый ConfigSettings
    let cfg = partial_cfg.into_final_settings();

    assert_eq!(cfg.log_level, "info");
    assert_eq!(
        cfg.remote_branch_template,
        "%owner%/%reponame%/%branchname%"
    );
    assert_eq!(cfg.source_remote_name, "origin");
    assert_eq!(cfg.remote_urls.len(), 2);
    assert_eq!(cfg.remote_urls["mirror1"], "git@github.com:owner/repo.git");
}

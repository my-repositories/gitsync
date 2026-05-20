use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;

use gitsync::configuration::config_settings::ConfigSettings;
use gitsync::services::known_hosts_service::KnownHostsService;
use gitsync::services::process_runner::IProcessRunner;
use tempfile::TempDir;

struct FakeRunner {
    calls: RefCell<Vec<(String, Vec<String>)>>,
    responses: RefCell<VecDeque<Result<String, String>>>,
}

impl FakeRunner {
    fn new(responses: Vec<Result<&str, &str>>) -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            responses: RefCell::new(
                responses
                    .into_iter()
                    .map(|r| r.map(|s| s.to_string()).map_err(|e| e.to_string()))
                    .collect(),
            ),
        }
    }
}

impl IProcessRunner for FakeRunner {
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<String, String> {
        self.calls.borrow_mut().push((
            file_name.to_string(),
            arguments.iter().map(|s| s.to_string()).collect(),
        ));

        self.responses
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| Err("unexpected call".to_string()))
    }
}

fn cfg() -> ConfigSettings {
    ConfigSettings {
        log_level: "Information".to_string(),
        source_remote_name: "origin".to_string(),
        remote_branch_template: "%owner%/%reponame%/%branchname%".to_string(),
        remote_urls: std::collections::HashMap::from([
            (
                "mirror1".to_string(),
                "git@github.com:owner/repo.git".to_string(),
            ),
            (
                "mirror2".to_string(),
                "https://gitlab.com/team/repo.git".to_string(),
            ),
        ]),
    }
}

#[tokio::test]
async fn warm_up_writes_known_hosts() {
    let tmp = TempDir::new().unwrap();
    let ssh_dir = tmp.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).unwrap();

    let known_hosts_path = ssh_dir.join("known_hosts");
    fs::write(&known_hosts_path, "oldhost.example.com ssh-ed25519 OLD\n").unwrap();

    let runner = FakeRunner::new(vec![
        Ok("github.com ssh-ed25519 AAAA\n"),
        Ok("gitlab.com ssh-ed25519 BBBB\n"),
    ]);

    let svc = KnownHostsService::new(runner, cfg());
    svc.warm_up_at(known_hosts_path.clone()).await.unwrap();

    let content = fs::read_to_string(&known_hosts_path).unwrap();
    assert!(content.contains("github.com ssh-ed25519 AAAA"));
    assert!(content.contains("gitlab.com ssh-ed25519 BBBB"));
    assert!(content.contains("oldhost.example.com ssh-ed25519 OLD"));
}

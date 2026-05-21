use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use gitsync::configuration::config_settings::ConfigSettings;
use gitsync::services::git_sync_service::GitSyncService;
use gitsync::services::process_runner::{IProcessRunner, ProcessOutput};

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
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<ProcessOutput, String> {
        self.calls.borrow_mut().push((
            file_name.to_string(),
            arguments.iter().map(|s| s.to_string()).collect(),
        ));

        let raw_response = self
            .responses
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| Err("unexpected call".to_string()));

        match raw_response {
            Ok(stdout) => Ok(ProcessOutput {
                stdout,
                stderr: "".to_string(),
                success: true,
            }),
            Err(stderr) => Ok(ProcessOutput {
                stdout: "".to_string(),
                stderr,
                success: false,
            }),
        }
    }
}

fn cfg() -> ConfigSettings {
    ConfigSettings {
        log_level: "info".to_string(),
        source_remote_name: "origin".to_string(),
        remote_branch_template: "%owner%/%reponame%/%branchname%".to_string(),
        ssh_key_types: "".to_string(),
        remote_urls: HashMap::from([(
            "mirror1".to_string(),
            "git@github.com:owner/repo.git".to_string(),
        )]),
    }
}

#[test]
fn run_success_path() {
    let runner = FakeRunner::new(vec![
        Ok("true"),                          // git rev-parse --is-inside-work-tree
        Ok("git@github.com:owner/repo.git"), // git remote get-url origin
        Ok("main"),                          // git branch --show-current
        Err("No such remote"),               // git remote get-url mirror1
        Ok(""),                              // git remote add mirror1 ...
        Ok(""),                              // git push mirror1 ...
    ]);

    let svc = GitSyncService::new(runner, cfg());
    svc.run().unwrap();
}

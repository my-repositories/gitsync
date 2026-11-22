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
    fn new(responses: Vec<Result<&'static str, &'static str>>) -> Self {
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

/// Helper: minimal response sequence for a successful push (no remote add needed).
fn success_responses() -> Vec<Result<&'static str, &'static str>> {
    vec![
        Ok("true"),                          // git rev-parse --is-inside-work-tree
        Ok("git@github.com:owner/repo.git"), // git remote get-url origin
        Ok("main"),                          // git branch --show-current
        Ok("git@github.com:owner/repo.git"), // git remote get-url mirror1 (already exists)
        Ok(""),                              // git push mirror1 ...
    ]
}

/// Helper: response sequence where push fails.
fn fail_push_responses() -> Vec<Result<&'static str, &'static str>> {
    vec![
        Ok("true"),                          // git rev-parse --is-inside-work-tree
        Ok("git@github.com:owner/repo.git"), // git remote get-url origin
        Ok("main"),                          // git branch --show-current
        Ok("git@github.com:owner/repo.git"), // git remote get-url mirror1
        Err("remote: Permission denied"),    // git push mirror1 ...
    ]
}

/// Helper: HTTPS remote URL for origin.
fn https_success_responses() -> Vec<Result<&'static str, &'static str>> {
    vec![
        Ok("true"),                              // git rev-parse --is-inside-work-tree
        Ok("https://github.com/owner/repo.git"), // git remote get-url origin
        Ok("main"),                              // git branch --show-current
        Ok("https://github.com/owner/repo.git"), // git remote get-url mirror1
        Ok(""),                                  // git push mirror1 ...
    ]
}

#[test]
fn run_success_path() {
    let runner = FakeRunner::new(success_responses());
    let svc = GitSyncService::new(runner, cfg());
    svc.run().unwrap();
}

#[test]
fn run_success_logs_info_no_errors() {
    let runner = FakeRunner::new(success_responses());
    let svc = GitSyncService::new(runner, cfg());

    let result = svc.run();

    assert!(result.is_ok(), "run should succeed when all pushes succeed");
}

#[test]
fn run_with_push_error_logs_warn() {
    let runner = FakeRunner::new(fail_push_responses());
    let svc = GitSyncService::new(runner, cfg());

    let result = svc.run();

    assert!(
        result.is_ok(),
        "run should not propagate individual push errors"
    );
}

#[test]
fn compute_summary_no_errors_returns_info_branch() {
    let runner = FakeRunner::new(success_responses());
    let svc = GitSyncService::new(runner, cfg());

    let results: Vec<(String, bool)> = svc.push_all("main:origin/main");
    let (success, error, total) = svc.compute_summary(&results);

    assert_eq!(total, 1);
    assert_eq!(success, 1);
    assert_eq!(error, 0);
}

#[test]
fn compute_summary_with_errors_returns_warn_branch() {
    let svc = GitSyncService::new(FakeRunner::new(vec![]), cfg());

    let results = vec![("mirror1".to_string(), false)];
    let (success, error, total) = svc.compute_summary(&results);

    assert_eq!(total, 1);
    assert_eq!(success, 0);
    assert_eq!(error, 1);
}

#[test]
fn compute_summary_multiple_remotes_mixed() {
    let svc = GitSyncService::new(FakeRunner::new(vec![]), cfg());

    let results = vec![
        ("mirror1".to_string(), true),
        ("mirror2".to_string(), false),
        ("mirror3".to_string(), true),
    ];
    let (success, error, total) = svc.compute_summary(&results);

    assert_eq!(total, 3);
    assert_eq!(success, 2);
    assert_eq!(error, 1);
}

#[test]
fn run_with_https_origin_succeeds() {
    let runner = FakeRunner::new(https_success_responses());
    let svc = GitSyncService::new(runner, cfg());

    let result = svc.run();
    assert!(result.is_ok(), "run should succeed with HTTPS origin URL");
}

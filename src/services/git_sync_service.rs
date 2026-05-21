use anyhow::{Result, anyhow};
use log::{debug, info};

use crate::configuration::config_settings::ConfigSettings;
use crate::services::process_runner::IProcessRunner;

pub struct GitSyncService<R: IProcessRunner> {
    process_runner: R,
    logger_cfg: ConfigSettings,
}

impl<R: IProcessRunner> GitSyncService<R> {
    pub fn new(process_runner: R, cfg: ConfigSettings) -> Self {
        Self {
            process_runner,
            logger_cfg: cfg,
        }
    }

    pub fn run(&self) -> Result<()> {
        self.ensure_inside_git_repo()?;

        info!("Start syncing");

        let refspec = self.build_refspec()?;
        self.ensure_remotes()?;

        let results = self.push_all(&refspec);
        self.log_summary(&results);

        Ok(())
    }

    fn ensure_remotes(&self) -> Result<()> {
        for (name, url) in &self.logger_cfg.remote_urls {
            self.ensure_remote(name, url)?;
        }
        Ok(())
    }

    fn push_all(&self, refspec: &str) -> Vec<(String, bool)> {
        self.logger_cfg
            .remote_urls
            .keys()
            .map(|name| {
                let ok = self.push_remote(name, refspec).is_ok();
                (name.clone(), ok)
            })
            .collect()
    }

    fn log_summary(&self, results: &[(String, bool)]) {
        let success = results.iter().filter(|(_, ok)| *ok).count();
        let error = results.len() - success;

        info!("Success total: {}/{}", success, results.len());
        info!("Error total: {}/{}", error, results.len());
    }

    fn build_refspec(&self) -> Result<String> {
        let source_remote_url = self
            .process_runner
            .run(
                "git",
                &["remote", "get-url", &self.logger_cfg.source_remote_name],
            )
            .map_err(anyhow::Error::msg)?;

        let branch_name = self
            .process_runner
            .run("git", &["branch", "--show-current"])
            .map_err(anyhow::Error::msg)?;

        let remote_branch = Self::build_remote_branch(
            &self.logger_cfg.remote_branch_template,
            &source_remote_url,
            &branch_name,
        )?;

        Ok(format!("{branch_name}:{remote_branch}"))
    }

    fn push_remote(&self, remote_name: &str, refspec: &str) -> Result<()> {
        self.process_runner
            .run("git", &["push", remote_name, refspec])
            .map_err(|e| anyhow!("{remote_name} failed to sync: {e}"))?;

        info!("{} push successfully completed", remote_name);
        Ok(())
    }

    fn ensure_remote(&self, remote_name: &str, remote_url: &str) -> Result<()> {
        let existing_url = self.try_get_remote_url(remote_name)?;

        match existing_url {
            None => {
                info!("{} adding remote url...", remote_name);
                self.process_runner
                    .run("git", &["remote", "add", remote_name, remote_url])
                    .map_err(|e| anyhow!("{remote_name} failed to add remote: {e}"))?;
                info!("{} remote url successfully added", remote_name);
            }
            Some(existing) if existing.eq_ignore_ascii_case(remote_url) => {
                debug!("{} remote url already exists, skipping", remote_name);
            }
            Some(_) => {
                info!("{} updating remote url...", remote_name);
                self.process_runner
                    .run("git", &["remote", "set-url", remote_name, remote_url])
                    .map_err(|e| anyhow!("{remote_name} failed to update remote: {e}"))?;
                info!("{} remote url successfully updated", remote_name);
            }
        }

        Ok(())
    }

    fn try_get_remote_url(&self, remote_name: &str) -> Result<Option<String>> {
        match self
            .process_runner
            .run("git", &["remote", "get-url", remote_name])
        {
            Ok(url) => Ok(Some(url)),
            Err(err) if err.contains("No such remote") => Ok(None),
            Err(err) => Err(anyhow!(err).context("Failed to get remote url")),
        }
    }

    fn ensure_inside_git_repo(&self) -> Result<()> {
        self.process_runner
            .run("git", &["rev-parse", "--is-inside-work-tree"])
            .map_err(|e| anyhow!("Current directory is not a git repository: {e}"))?;
        Ok(())
    }

    fn build_remote_branch(template: &str, origin_url: &str, branch_name: &str) -> Result<String> {
        let repo_part = if let Some(pos) = origin_url.find(':') {
            &origin_url[pos + 1..]
        } else {
            origin_url
        };

        let repo_part = repo_part.strip_suffix(".git").unwrap_or(repo_part);
        let mut parts = repo_part.splitn(2, '/');

        let owner = parts.next().unwrap_or("");
        let reponame = parts.next().unwrap_or("");

        if owner.is_empty() || reponame.is_empty() {
            return Err(anyhow!("Invalid origin url: {origin_url}"));
        }

        Ok(template
            .replace("%owner%", owner)
            .replace("%reponame%", reponame)
            .replace("%branchname%", branch_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

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
            log_level: "info".to_string(),
            source_remote_name: "origin".to_string(),
            remote_branch_template: "%owner%/%reponame%/%branchname%".to_string(),
            remote_urls: std::collections::HashMap::from([(
                "mirror1".to_string(),
                "git@github.com:owner/repo.git".to_string(),
            )]),
        }
    }

    #[test]
    fn build_remote_branch_works() {
        let out = GitSyncService::<FakeRunner>::build_remote_branch(
            "%owner%/%reponame%/%branchname%",
            "git@github.com:alice/project.git",
            "main",
        )
        .unwrap();

        assert_eq!(out, "alice/project/main");
    }

    #[test]
    fn build_remote_branch_rejects_invalid_url() {
        let err = GitSyncService::<FakeRunner>::build_remote_branch(
            "%owner%/%reponame%/%branchname%",
            "invalid",
            "main",
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("Invalid origin url"));
    }

    #[test]
    fn ensure_inside_git_repo_fails_when_git_says_no() {
        let runner = FakeRunner::new(vec![Err("not a repo")]);
        let svc = GitSyncService::new(runner, cfg());

        let err = svc.ensure_inside_git_repo().unwrap_err().to_string();
        assert!(err.contains("Current directory is not a git repository"));
    }

    #[test]
    fn ensure_remote_adds_missing_remote() {
        let runner = FakeRunner::new(vec![Err("No such remote"), Ok("")]);
        let svc = GitSyncService::new(runner, cfg());

        svc.ensure_remote("mirror1", "git@github.com:owner/repo.git")
            .unwrap();
    }

    #[test]
    fn ensure_remote_updates_different_url() {
        let runner = FakeRunner::new(vec![Ok("git@github.com:old/repo.git"), Ok("")]);
        let svc = GitSyncService::new(runner, cfg());

        svc.ensure_remote("mirror1", "git@github.com:owner/repo.git")
            .unwrap();
    }

    #[test]
    fn push_remote_runs_git_push() {
        let runner = FakeRunner::new(vec![Ok("")]);
        let svc = GitSyncService::new(runner, cfg());

        svc.push_remote("mirror1", "main:alice/project/main")
            .unwrap();
    }

    #[test]
    fn try_get_remote_url_returns_none_for_missing_remote() {
        let runner = FakeRunner::new(vec![Err("No such remote")]);
        let svc = GitSyncService::new(runner, cfg());

        let url = svc.try_get_remote_url("mirror1").unwrap();
        assert!(url.is_none());
    }
}

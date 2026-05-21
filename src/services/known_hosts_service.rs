use std::{env, path::PathBuf};

use anyhow::Result;
use log::debug;
use tokio::fs;

use crate::configuration::config_settings::ConfigSettings;
use crate::services::process_runner::IProcessRunner;

pub struct KnownHostsService<R: IProcessRunner> {
    process_runner: R,
    cfg: ConfigSettings,
}

impl<R: IProcessRunner> KnownHostsService<R> {
    pub fn new(process_runner: R, cfg: ConfigSettings) -> Self {
        Self {
            process_runner,
            cfg,
        }
    }

    pub async fn warm_up_at(&self, known_hosts_path: PathBuf) -> Result<()> {
        let hosts = self.get_hosts()?;

        debug!("Known hosts warmup started for: {:?}", hosts);

        let existing = self.load_known_hosts(&known_hosts_path).await?;
        let updated = self.build_updated_known_hosts(existing, &hosts).await?;

        self.save_known_hosts(&known_hosts_path, &updated).await?;

        debug!(
            "Known hosts warmup completed. Total lines: {}",
            updated.len()
        );
        Ok(())
    }

    pub async fn warm_up(&self) -> Result<()> {
        let known_hosts_path = Self::get_known_hosts_path();
        self.warm_up_at(known_hosts_path).await
    }

    fn get_hosts(&self) -> Result<Vec<String>> {
        let mut hosts = self
            .cfg
            .remote_urls
            .values()
            .map(|url| Self::parse_host(url))
            .collect::<Result<Vec<_>>>()?;

        hosts.sort();
        hosts.dedup();
        Ok(hosts)
    }

    fn get_known_hosts_path() -> PathBuf {
        let home = env::var("HOME").ok();
        Self::resolve_known_hosts_path(home.as_deref())
    }

    fn resolve_known_hosts_path(home: Option<&str>) -> PathBuf {
        let home = home.unwrap_or(".");
        let ssh_dir = PathBuf::from(home).join(".ssh");
        let _ = std::fs::create_dir_all(&ssh_dir);
        ssh_dir.join("known_hosts")
    }

    async fn load_known_hosts(&self, known_hosts_path: &PathBuf) -> Result<Vec<String>> {
        if !known_hosts_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(known_hosts_path).await?;
        Ok(content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    async fn build_updated_known_hosts(
        &self,
        existing: Vec<String>,
        hosts: &[String],
    ) -> Result<Vec<String>> {
        let mut preserved: Vec<String> = existing
            .into_iter()
            .filter(|line| !hosts.iter().any(|host| Self::line_matches_host(line, host)))
            .collect();

        let scanned = self.scan_hosts(hosts).await?;
        preserved.extend(scanned);
        preserved.sort();
        preserved.dedup();

        Ok(preserved)
    }

    async fn scan_hosts(&self, hosts: &[String]) -> Result<Vec<String>> {
        let mut result = Vec::new();

        for host in hosts {
            debug!("Scanning host {}", host);

            let mut args = Vec::new();
            if !self.cfg.ssh_key_types.trim().is_empty() {
                args.push("-t");
                args.push(self.cfg.ssh_key_types.trim());
            }
            args.push(host);

            let output = match self
                .process_runner
                .run("ssh-keyscan", &args) 
            {
                Ok(out) => out,
                Err(err_msg) => {
                    log::warn!("ssh-keyscan failed for host {}: {}", host, err_msg);
                    continue;
                }
            };

            let lines = output
                .lines()
                .map(str::trim)
                .filter(|line| Self::line_is_valid_known_host_line(line))
                .filter(|line| Self::line_matches_host(line, host))
                .map(ToOwned::to_owned);

            result.extend(lines);
        }

        Ok(result)
    }

    async fn save_known_hosts(&self, known_hosts_path: &PathBuf, lines: &[String]) -> Result<()> {
        if lines.is_empty() {
            fs::write(known_hosts_path, "").await?;
            return Ok(());
        }

        let content = lines.join("\n") + "\n";
        fs::write(known_hosts_path, content).await?;
        Ok(())
    }

    fn line_is_valid_known_host_line(line: &str) -> bool {
        !line.trim().is_empty() && !line.starts_with('#')
    }

    fn line_matches_host(line: &str, host: &str) -> bool {
        line.starts_with(&(host.to_string() + " ")) || line.starts_with(&(host.to_string() + ","))
    }

    fn parse_host(remote_url: &str) -> Result<String> {
        if remote_url.starts_with("git@") && remote_url.contains(':') {
            let after_at = remote_url.split('@').nth(1).unwrap_or(remote_url);
            let host = after_at.split(':').next().unwrap_or("");
            if !host.is_empty() {
                return Ok(host.to_string());
            }
        }

        if let Ok(url) = url::Url::parse(remote_url)
            && let Some(host) = url.host_str()
        {
            return Ok(host.to_string());
        }

        anyhow::bail!("Invalid remote url: {remote_url}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};

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
            ssh_key_types: "".to_string(),
            remote_urls: HashMap::from([
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

    #[test]
    fn parse_host_git_ssh_url() {
        assert_eq!(
            KnownHostsService::<FakeRunner>::parse_host("git@github.com:owner/repo.git").unwrap(),
            "github.com"
        );
    }

    #[test]
    fn parse_host_https_url() {
        assert_eq!(
            KnownHostsService::<FakeRunner>::parse_host("https://gitlab.com/team/repo.git")
                .unwrap(),
            "gitlab.com"
        );
    }

    #[test]
    fn parse_host_rejects_invalid_url() {
        let err = KnownHostsService::<FakeRunner>::parse_host("nope")
            .unwrap_err()
            .to_string();
        assert!(err.contains("Invalid remote url"));
    }

    #[test]
    fn line_is_valid_known_host_line_works() {
        assert!(
            KnownHostsService::<FakeRunner>::line_is_valid_known_host_line(
                "github.com ssh-ed25519 AAAA"
            )
        );
        assert!(!KnownHostsService::<FakeRunner>::line_is_valid_known_host_line("# comment"));
        assert!(!KnownHostsService::<FakeRunner>::line_is_valid_known_host_line("   "));
    }

    #[test]
    fn line_matches_host_works() {
        assert!(KnownHostsService::<FakeRunner>::line_matches_host(
            "github.com ssh-ed25519 AAAA",
            "github.com"
        ));
        assert!(KnownHostsService::<FakeRunner>::line_matches_host(
            "github.com,1.2.3.4 ssh-ed25519 AAAA",
            "github.com"
        ));
        assert!(!KnownHostsService::<FakeRunner>::line_matches_host(
            "gitlab.com ssh-ed25519 AAAA",
            "github.com"
        ));
    }

    #[test]
    fn get_hosts_dedup_and_sort() {
        let runner = FakeRunner::new(vec![]);
        let svc = KnownHostsService::new(runner, cfg());

        let hosts = svc.get_hosts().unwrap();
        assert_eq!(
            hosts,
            vec!["github.com".to_string(), "gitlab.com".to_string()]
        );
    }
}

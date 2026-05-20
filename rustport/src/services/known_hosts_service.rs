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
        Self { process_runner, cfg }
    }

    pub async fn warm_up(&self) -> Result<()> {
        let hosts = self.get_hosts()?;
        let known_hosts_path = Self::get_known_hosts_path();

        debug!("Known hosts warmup started for: {:?}", hosts);

        let existing = self.load_known_hosts(&known_hosts_path).await?;
        let updated = self.build_updated_known_hosts(existing, &hosts).await?;

        self.save_known_hosts(&known_hosts_path, &updated).await?;

        debug!("Known hosts warmup completed. Total lines: {}", updated.len());
        Ok(())
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
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
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

            let output = self
                .process_runner
                .run("ssh-keyscan", &[host])
                .map_err(anyhow::Error::msg)?;

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
        line.starts_with(&(host.to_string() + " "))
            || line.starts_with(&(host.to_string() + ","))
    }

    fn parse_host(remote_url: &str) -> Result<String> {
        if remote_url.starts_with("git@") && remote_url.contains(':') {
            let after_at = remote_url.split('@').nth(1).unwrap_or(remote_url);
            let host = after_at.split(':').next().unwrap_or("");
            if !host.is_empty() {
                return Ok(host.to_string());
            }
        }

        if let Ok(url) = url::Url::parse(remote_url) {
            if let Some(host) = url.host_str() {
                return Ok(host.to_string());
            }
        }

        anyhow::bail!("Invalid remote url: {remote_url}");
    }
}
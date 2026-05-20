mod configuration;
mod services;

use anyhow::Result;
use configuration::config_reader::ConfigReader;
use log::LevelFilter;
use services::git_sync_service::GitSyncService;
use services::known_hosts_service::KnownHostsService;
use services::process_runner::ProcessRunner;

fn init_logging(log_level: &str) -> Result<()> {
    let level = log_level
        .parse::<LevelFilter>()
        .map_err(|_| anyhow::anyhow!("Invalid log level: {log_level}"))?;

    env_logger::Builder::new()
        .filter_level(level)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = ConfigReader::read_config().await?;
    init_logging(&cfg.log_level)?;

    let process_runner = ProcessRunner;
    let known_hosts_service = KnownHostsService::new(process_runner, cfg.clone());
    let git_sync_service = GitSyncService::new(process_runner, cfg);

    let skip_host = std::env::args().any(|a| {
        a.eq_ignore_ascii_case("--skip-host") || a.eq_ignore_ascii_case("-s")
    });

    if !skip_host {
        known_hosts_service.warm_up().await?;
    }

    git_sync_service.run()?;
    Ok(())
}
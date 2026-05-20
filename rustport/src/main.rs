mod configuration;
mod services;

use anyhow::Result;
use configuration::config_reader::ConfigReader;
use log::LevelFilter;
use services::git_sync_service::GitSyncService;
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
    let git_sync_service = GitSyncService::new(process_runner, cfg);

    git_sync_service.run()?;
    Ok(())
}
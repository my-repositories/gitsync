mod configuration;
mod services;

use anyhow::Result;
use configuration::config_reader::ConfigReader;
use services::git_sync_service::GitSyncService;
use services::process_runner::ProcessRunner;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = ConfigReader::read_config().await?;
    let process_runner = ProcessRunner;
    let gitSyncService = GitSyncService::new(process_runner, cfg);

    gitSyncService.run()?;
    Ok(())
}
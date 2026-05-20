use anyhow::Result;
use gitsync::configuration::config_reader::ConfigReader;
use gitsync::services::git_sync_service::GitSyncService;
use gitsync::services::known_hosts_service::KnownHostsService;
use gitsync::services::process_runner::ProcessRunner;
use log::LevelFilter;

fn init_logging(log_level: &str) -> Result<()> {
    let level = log_level
        .parse::<LevelFilter>()
        .map_err(|_| anyhow::anyhow!("Invalid log level: {log_level}"))?;

    env_logger::Builder::new().filter_level(level).init();
    Ok(())
}

fn should_skip_host(args: &[String]) -> bool {
    args.iter()
        .any(|a| a.eq_ignore_ascii_case("--skip-host") || a.eq_ignore_ascii_case("-s"))
}

async fn run_app(skip_host: bool) -> Result<()> {
    let cfg = ConfigReader::read_config().await?;
    init_logging(&cfg.log_level)?;

    let process_runner = ProcessRunner;
    let known_hosts_service = KnownHostsService::new(process_runner, cfg.clone());
    let git_sync_service = GitSyncService::new(process_runner, cfg);

    if !skip_host {
        known_hosts_service.warm_up().await?;
    }

    git_sync_service.run()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let skip_host = should_skip_host(&args);
    run_app(skip_host).await
}

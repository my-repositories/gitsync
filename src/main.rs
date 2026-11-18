use anyhow::Result;
use gitsync::configuration::config_reader::ConfigReader;
use gitsync::services::git_sync_service::GitSyncService;
use gitsync::services::known_hosts_service::KnownHostsService;
use gitsync::services::process_runner::ProcessRunner;
use log::LevelFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_logging(log_level: &str) -> Result<()> {
    let level = log_level
        .parse::<LevelFilter>()
        .map_err(|_| anyhow::anyhow!("Invalid log level: {log_level}"))?;

    env_logger::Builder::new().filter_level(level).init();
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliAction {
    Run,
    Version,
    Update,
    Help,
}

fn parse_args() -> CliAction {
    let args: Vec<String> = std::env::args().collect();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-v" | "--version" => return CliAction::Version,
            "-u" | "--update" => return CliAction::Update,
            "-h" | "--help" => return CliAction::Help,
            _ => {}
        }
    }
    CliAction::Run
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

fn check_for_update() -> Result<()> {
    use std::env;
    use std::process::Command;

    println!("gitsync v{VERSION}");
    println!("Checking for updates...");

    // Determine platform
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let url = "https://api.github.com/repos/my-repositories/gitsync/releases/latest";

    // Fetch latest release info
    let output = Command::new("curl")
        .args(["-s", "-H", "Accept: application/vnd.github+json", url])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run curl: {e}"))?;

    if !output.status.success() {
        eprintln!("Failed to fetch release info");
        std::process::exit(1);
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // Parse version from JSON response
    let latest_version = response
        .find(r#""tag_name":"#)
        .and_then(|pos| {
            let after = &response[pos + r#""tag_name":"#.len()..];
            after
                .trim_start()
                .strip_prefix('"')
                .and_then(|v| v.split('"').next())
        })
        .unwrap_or(VERSION);

    let current_version = VERSION.trim_start_matches('v');
    let latest_version = latest_version.trim_start_matches('v');

    if current_version == latest_version {
        println!("Already up to date (v{VERSION})");
        return Ok(());
    }

    println!("New version available: v{latest_version} (current: v{VERSION})");

    // Find the asset for this platform
    let asset_url = response
        .find(r#""browser_download_url":"#)
        .map(|i| response[i + 23..].trim_start())
        .and_then(|s| s.split('"').next())
        .unwrap_or("");

    if asset_url.is_empty() {
        eprintln!(
            "Could not find download URL for this platform ({}-{})",
            os, arch
        );
        std::process::exit(1);
    }

    println!("Downloading update...");

    let current_exe = env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {e}"))?;
    let current_exe_str = current_exe
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Current executable path is not valid UTF-8"))?;

    // Download and replace
    let status = Command::new("curl")
        .args(["-L", "-o", current_exe_str, asset_url])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run curl download: {e}"))?;

    if !status.success() {
        eprintln!("Download failed");
        std::process::exit(1);
    }

    // Make executable
    #[cfg(unix)]
    Command::new("chmod")
        .args(["+x", current_exe_str])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to set executable permission: {e}"))?;

    println!("Updated to v{latest_version}");
    Ok(())
}

fn print_help() {
    println!("gitsync v{VERSION}");
    println!();
    println!("USAGE:");
    println!("    gitsync [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help          Print help");
    println!("    -v, --version       Print version");
    println!("    -u, --update        Check for and apply updates");
    println!("    -s, --skip-host     Skip SSH known_hosts warm-up");
    println!();
    println!("COMMANDS:");
    println!("    gitsync             Run git sync (default)");
}

#[tokio::main]
async fn main() -> Result<()> {
    let action = parse_args();

    match action {
        CliAction::Version => {
            println!("gitsync v{VERSION}");
            Ok(())
        }
        CliAction::Update => check_for_update(),
        CliAction::Help => {
            print_help();
            Ok(())
        }
        CliAction::Run => {
            let args: Vec<String> = std::env::args().collect();
            let skip_host = args
                .iter()
                .any(|a| a.eq_ignore_ascii_case("--skip-host") || a.eq_ignore_ascii_case("-s"));
            run_app(skip_host).await
        }
    }
}

use gitsync::configuration::config_reader::ConfigReader;
use gitsync::services::git_sync_service::GitSyncService;
use gitsync::services::known_hosts_service::KnownHostsService;
use gitsync::services::process_runner::ProcessRunner;
use log::LevelFilter;

fn should_skip_host(args: &[String]) -> bool {
    args.iter()
        .any(|a| a.eq_ignore_ascii_case("--skip-host") || a.eq_ignore_ascii_case("-s"))
}

fn parse_level(log_level: &str) -> Result<LevelFilter, String> {
    log_level
        .parse::<LevelFilter>()
        .map_err(|_| format!("Invalid log level: {log_level}"))
}

#[test]
fn should_skip_host_detects_long_flag() {
    let args = vec!["app".to_string(), "--skip-host".to_string()];
    assert!(should_skip_host(&args));
}

#[test]
fn should_skip_host_detects_short_flag() {
    let args = vec!["app".to_string(), "-s".to_string()];
    assert!(should_skip_host(&args));
}

#[test]
fn should_skip_host_returns_false_without_flag() {
    let args = vec!["app".to_string()];
    assert!(!should_skip_host(&args));
}

#[test]
fn parse_level_accepts_valid_level() {
    assert_eq!(parse_level("info").unwrap(), LevelFilter::Info);
}

#[test]
fn parse_level_rejects_invalid_level() {
    let err = parse_level("nope").unwrap_err();
    assert!(err.contains("Invalid log level"));
}

#[test]
fn main_types_are_usable() {
    let _ = ConfigReader::read_config;
    let _ = GitSyncService::<ProcessRunner>::new;
    let _ = KnownHostsService::<ProcessRunner>::new;
}

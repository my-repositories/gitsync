mod configuration;

use anyhow::Result;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let output = Command::new("git")
        .arg("status")
        .output()
        .expect("failed to run git status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        println!("{}", stdout);
    } else {
        eprintln!("{}", stderr);
    }

    Ok(())
}
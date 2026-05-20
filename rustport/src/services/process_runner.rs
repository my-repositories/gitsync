use std::process::Command;

pub struct ProcessRunner;

impl ProcessRunner {
    pub fn run(&self, file_name: &str, arguments: &[&str]) -> Result<String, String> {
        let output = Command::new(file_name)
            .args(arguments)
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            Err(if stderr.is_empty() { stdout } else { stderr })
        }
    }
}
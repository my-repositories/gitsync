use log::debug;
use std::process::Command;

pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub trait IProcessRunner {
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<ProcessOutput, String>;
}

#[derive(Clone, Copy)]
pub struct ProcessRunner;

impl IProcessRunner for ProcessRunner {
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<ProcessOutput, String> {
        debug!("Executing command: {} with args: {:?}", file_name, arguments);

        let output = Command::new(file_name)
            .args(arguments)
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !stdout.is_empty() {
            debug!("[{}] STDOUT:\n{}", file_name, stdout);
        }
        if !stderr.is_empty() {
            debug!("[{}] STDERR:\n{}", file_name, stderr);
        }

        Ok(ProcessOutput {
            stdout,
            stderr,
            success: output.status.success(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_cmd() -> (&'static str, &'static str) {
        if cfg!(windows) { ("cmd", "/c") } else { ("sh", "-c") }
    }

    #[test]
    fn returns_stdout_on_success() {
        let runner = ProcessRunner;
        let (shell, flag) = shell_cmd();
        let result = runner.run(shell, &[flag, "echo hello"]).unwrap();
        assert_eq!(result.stdout, "hello");
        assert!(result.success);
    }

    #[test]
    fn returns_stderr_on_failure() {
        let runner = ProcessRunner;
        let (shell, flag) = shell_cmd();
        let cmd = if cfg!(windows) { "echo error>&2 & exit 1" } else { "printf error 1>&2; exit 1" };
        let result = runner.run(shell, &[flag, cmd]).unwrap();
        assert_eq!(result.stderr, "error");
        assert!(!result.success);
    }
}

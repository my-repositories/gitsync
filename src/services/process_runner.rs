use std::process::Command;

pub trait IProcessRunner {
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<String, String>;
}

#[derive(Clone, Copy)]
pub struct ProcessRunner;

impl IProcessRunner for ProcessRunner {
    fn run(&self, file_name: &str, arguments: &[&str]) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_stdout_on_success() {
        let runner = ProcessRunner;
        let result = runner.run("sh", &["-c", "printf hello"]);
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn returns_stderr_on_failure() {
        let runner = ProcessRunner;
        let result = runner.run("sh", &["-c", "printf error 1>&2; exit 1"]);
        assert_eq!(result.unwrap_err(), "error");
    }

    #[test]
    fn returns_stdout_when_failure_has_no_stderr() {
        let runner = ProcessRunner;
        let result = runner.run("sh", &["-c", "printf fallback; exit 1"]);
        assert_eq!(result.unwrap_err(), "fallback");
    }
}

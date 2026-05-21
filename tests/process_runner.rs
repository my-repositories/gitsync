use gitsync::services::process_runner::{IProcessRunner, ProcessRunner};

#[test]
fn returns_stdout_on_success() {
    let runner = ProcessRunner;
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let flag = if cfg!(windows) { "/c" } else { "-c" };

    let result = runner.run(shell, &[flag, "echo hello"]).unwrap();

    assert_eq!(result.stdout, "hello");
    assert!(result.success);
}

#[test]
fn returns_stderr_on_failure() {
    let runner = ProcessRunner;
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let flag = if cfg!(windows) { "/c" } else { "-c" };
    let cmd = if cfg!(windows) {
        "echo error>&2 & exit 1"
    } else {
        "printf error 1>&2; exit 1"
    };

    let result = runner.run(shell, &[flag, cmd]).unwrap();

    assert_eq!(result.stderr, "error");
    assert!(!result.success);
}

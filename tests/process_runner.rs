use gitsync::services::process_runner::{IProcessRunner, ProcessRunner};

#[test]
fn runs_command_successfully() {
    let runner = ProcessRunner;
    let result = runner.run("sh", &["-c", "printf hello"]);
    assert_eq!(result.unwrap(), "hello");
}

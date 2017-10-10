extern crate criterion;

use criterion::Criterion;
use std::process::Command;

fn create_command() -> Command {
    let mut command = Command::new("python3");
    command.arg("benches/external_process.py");
    command
}

#[test]
fn python_fibonacci() {
    let has_python3 = Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output().is_ok();

    if has_python3 {
        Criterion::default()
            .bench_program("fibonacci-python", create_command());
    }
}

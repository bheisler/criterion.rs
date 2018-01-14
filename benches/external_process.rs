use criterion::Criterion;
use std::process::Command;
use std::process::Stdio;

fn create_command() -> Command {
    let mut command = Command::new("python3");
    command.arg("benches/external_process.py");
    command
}

fn python_fibonacci(c: &mut Criterion) {
    let has_python3 = Command::new("python3")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output().is_ok();

    if has_python3 {
        c.bench_program("fibonacci-python", create_command());
    }
}

criterion_group!(benches, python_fibonacci);
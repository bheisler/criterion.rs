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
    Criterion::default()
        .bench_program_over_inputs("fibonacci-python",
        create_command,
        &[1, 2, 4, 8, 16]);
}

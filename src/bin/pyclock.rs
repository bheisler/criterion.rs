extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    let mut cmd = Command::new("python3");
    cmd.args(["-O", "-u", "external/clock.py"]);

    Criterion::default().
        ext_bench("python/clock", cmd);
}

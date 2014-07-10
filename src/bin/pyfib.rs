extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    let mut cmd = Command::new("python3");
    cmd.args(["-O", "-u", "external/fib.py"]);

    Criterion::default().
        ext_bench_group("python/fib", [5u, 10, 15], cmd);
}

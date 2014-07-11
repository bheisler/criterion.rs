extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench_prog_family(
            "python/fib",
            Command::new("python3").args(["-O", "-u", "external/fib.py"]),
            [5u, 10, 15]);
}

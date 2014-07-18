extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench_prog_family(
            "python/fib",
            Command::new("python").args(["-O", "external/fib.py"]),
            [5u, 10, 15]);
}

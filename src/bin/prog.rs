extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench_prog(
            "python/clock",
            Command::new("python3").args(["-O", "external/clock.py"]));
}

extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench_prog("fib", &Command::new("target/release/fib"));
}

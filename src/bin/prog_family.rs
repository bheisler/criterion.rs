extern crate criterion;

use std::io::Command;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench_prog_family(
            "from_elem",
            &Command::new("target/release/from_elem"),
            [10u, 100, 1_000]);
}

extern crate criterion;

use criterion::{Bencher,Criterion};

fn fibonacci(n: uint) -> uint {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let mut b = Criterion::new();

    b.bench_group("fib", &[5u, 10, 15], fib);
}

fn fib(b: &mut Bencher, n: uint) {
    b.iter(|| {
        fibonacci(n)
    })
}

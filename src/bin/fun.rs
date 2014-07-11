extern crate criterion;

use criterion::{Bencher,Criterion};

fn fibonacci(n: uint) -> uint {
    if n > 1 { fibonacci(n - 1) + fibonacci(n - 2) } else { n + 1 }
}

fn main() {
    Criterion::default().bench("fib", fib);
}

fn fib(b: &mut Bencher) {
    b.iter(|| fibonacci(15))
}

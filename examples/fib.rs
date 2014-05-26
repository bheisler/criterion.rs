extern crate criterion;

use criterion::Bencher;

fn fib(n: uint) -> uint {
    match n {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
    let mut b = Bencher::new();

    b.bench_group("fib", &[5u, 10, 15], |n| fib(n));
}

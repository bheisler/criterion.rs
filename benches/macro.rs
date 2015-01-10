#![feature(plugin)]

extern crate criterion;
#[plugin]
extern crate criterion_macros;

use criterion::Bencher;

#[criterion]
fn fib5(b: &mut Bencher) {
    b.iter(|| fib(5))
}

fn fib(n: usize) -> usize {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

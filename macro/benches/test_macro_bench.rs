#![feature(custom_test_frameworks)]
#![test_runner(criterion::runner)]

use std::hint::black_box;
use criterion::Criterion;
use criterion_macro::criterion;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(50)
}

#[criterion]
fn bench_simple(c: &mut Criterion) {
    c.bench_function("Fibonacci-Simple", |b| b.iter(|| fibonacci(black_box(10))));
}

#[criterion(custom_criterion())]
fn bench_custom(c: &mut Criterion) {
    c.bench_function("Fibonacci-Custom", |b| b.iter(|| fibonacci(black_box(20))));
}

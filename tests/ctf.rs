//! Custom test framework support
#![cfg_attr(feature = "ctf", feature(custom_test_frameworks, proc_macro_gen))]
#![cfg_attr(feature = "ctf", test_runner(criterion::runner))]

#[cfg(feature = "ctf")]
extern crate criterion;
#[cfg(feature = "ctf")]
use criterion::*;

#[cfg(feature = "ctf")]
fn fibonacci_slow(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

#[cfg(feature = "ctf")]
fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut c: u64;

    if n == 0 {
        return 0;
    }

    for _ in 0..(n + 1) {
        c = a + b;
        a = b;
        b = c;
    }
    b
}

#[cfg(feature = "ctf")]
#[criterion]
fn compare_fibonaccis(c: &mut Criterion) {
    let fib_slow = Fun::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)));
    let fib_fast = Fun::new("Iterative", |b, i| b.iter(|| fibonacci_fast(*i)));

    let functions = vec![fib_slow, fib_fast];

    c.bench_functions("Fibonacci", functions, 20);
}

# Migrating from libtest

This page shows an example of converting a libtest or bencher benchmark to use
Criterion.rs.

## The Benchmark

We'll start with this benchmark as an example:

```rust
#![feature(test)]
extern crate test;
use test::Bencher;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

#[bench]
fn bench_fib(b: &mut Bencher) {
    b.iter(|| fibonacci(20));
}
```

## The Migration

The first thing to do is update the `Cargo.toml` to disable the libtest
benchmark harness:

```toml
[[bench]]
name = "example"
harness = false
```

We also need to add Criterion.rs to the `dev-dependencies` section of `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.2"
```

The next step is to update the imports:

```rust
#[macro_use]
extern crate criterion;
use criterion::Criterion;
```

Then, we can change the `bench_fib` function. Remove the `#[bench]` and change
the argument to `&mut Criterion` instead. The contents of this function need to
change as well:

```rust
fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}
```

Finally, we need to invoke some macros to generate a main function, since we
no longer have libtest to provide one:

```rust
criterion_group!(benches, bench_fib);
criterion_main!(benches);
```

And that's it! The complete migrated benchmark code is below:

```rust
#[macro_use]
extern crate criterion;
use criterion::Criterion;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}

criterion_group!(benches, bench_fib);
criterion_main!(benches);
```

# Bencher Compatibility Layer

Criterion.rs provides a small crate which can be used as a drop-in replacement for most common
usages of `bencher` in order to make it easy for existing `bencher` users to try out Criterion.rs.
This page shows an example of how to use this crate.

## Example

We'll start with the example benchmark from `bencher`:

```rust
use bencher::{benchmark_group, benchmark_main, Bencher};

fn a(bench: &mut Bencher) {
    bench.iter(|| {
        (0..1000).fold(0, |x, y| x + y)
    })
}

fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| {
        vec![0u8; N]
    });

    bench.bytes = N as u64;
}

benchmark_group!(benches, a, b);
benchmark_main!(benches);
```

The first step is to edit the Cargo.toml file to replace the bencher dependency with 
`criterion_bencher_compat`:

Change: 

```toml
[dev-dependencies]
bencher = "0.1"
```

To:

```toml
[dev-dependencies]
criterion_bencher_compat = "0.4"
```

Then we update the benchmark file itself to change:

```rust
use bencher::{benchmark_group, benchmark_main, Bencher};
```

To:

```rust
use criterion_bencher_compat as bencher;
use bencher::{benchmark_group, benchmark_main, Bencher};
```

That's all! Now just run `cargo bench`:

```text
     Running target/release/deps/bencher_example-d865087781455bd5
a                       time:   [234.58 ps 237.68 ps 241.94 ps]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

b                       time:   [23.972 ns 24.218 ns 24.474 ns]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
```

## Limitations

`criterion_bencher_compat` does not implement the full API of the `bencher` crate, only the most
commonly-used subset. If your benchmarks require parts of the `bencher` crate which are not 
supported, you may need to temporarily disable them while trying Criterion.rs.

`criterion_bencher_compat` does not provide access to most of Criterion.rs' more advanced features.
If the Criterion.rs benchmarks work well for you, it is recommended to convert your benchmarks to
use the Criterion.rs interface directly. See [Migrating from libtest](./migrating_from_libtest.md)
for more information on that.

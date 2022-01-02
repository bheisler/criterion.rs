## Getting Started

Iai is designed to be similar in interface to Criterion.rs, so using it is easy. To get started,
add the following to your Cargo.toml file:

```toml
[dev-dependencies]
iai = "0.1"

[[bench]]
name = "my_benchmark"
harness = false
```

Next, define a benchmark by creating a file at `$PROJECT/benches/my_benchmark.rs` with the following contents:

```rust
use iai::{black_box, main};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn iai_benchmark_short() -> u64 {
    fibonacci(black_box(10))
}

fn iai_benchmark_long() -> u64 {
    fibonacci(black_box(30))
}


iai::main!(iai_benchmark_short, iai_benchmark_long);
```

Finally, run this benchmark with `cargo bench`. You should see output similar to the following:

```
     Running target/release/deps/test_regular_bench-8b173c29ce041afa

bench_fibonacci_short
  Instructions:                1735
  L1 Accesses:                 2364
  L2 Accesses:                    1
  RAM Accesses:                   1
  Estimated Cycles:            2404

bench_fibonacci_long
  Instructions:            26214735
  L1 Accesses:             35638623
  L2 Accesses:                    2
  RAM Accesses:                   1
  Estimated Cycles:        35638668
```

<h1 align="center">Criterion.<span></span>rs</h1>

<div align="center">Statistics-driven Microbenchmarking in Rust</div>

<div align="center">
	<a href="https://japaric.github.io/criterion.rs/book/getting_started.html">Getting Started</a>
    |
    <a href="https://japaric.github.io/criterion.rs/book/index.html">User Guide</a>
    |
    <a href="https://japaric.github.io/criterion.rs/criterion/">API Documentation</a>
    |
    <a href="https://github.com/japaric/criterion.rs/blob/master/CHANGELOG.md">Changelog</a>
</div>

<div align="center">
	<a href="https://travis-ci.org/japaric/criterion.rs">
    	<img src="https://travis-ci.org/japaric/criterion.rs.svg?branch=master" alt="Travis-CI">
    </a>
    |
    <a href="https://ci.appveyor.com/project/bheisler/criterion-rs-vt9fl">
    	<img src="https://ci.appveyor.com/api/projects/status/4255ads9ctpupcl2?svg=true" alt="Appveyor">
    </a>
    |
    <a href="https://crates.io/crates/criterion">
        <img src="https://img.shields.io/crates/v/criterion.svg" alt="Crates.io">
    </a>
</div>

Criterion.<span></span>rs helps you write fast code by detecting and measuring performance improvements or regressions, even small ones, quickly and accurately. You can optimize with confidence, knowing how each change affects the performance of your code.

## Table of Contents
- [Features](#features)
- [Quickstart](#quickstart)
- [Goals](#goals)
- [Contributing](#contributing)
- [Maintenance](#maintenance)
- [License](#license)
- [Related Projects](#related-projects)

### Features

- __Statistics__: Statistical analysis detects if, and by how much, performance has changed since the last benchmark run
- __Charts__: Uses [gnuplot](http://www.gnuplot.info/) to generate detailed graphs of benchmark results.
- __Stable-compatible__: Benchmark your code without installing Nightly Rust.
- Benchmark external programs written in any language.

### Quickstart

Criterion.<span></span>rs currently requires a nightly version of Rust. Additionally, in order to generate plots, you must have [gnuplot](http://www.gnuplot.info/) installed. See the gnuplot website for installation instructions.

To start with Criterion.<span></span>rs, add the following to your `cargo.toml` file:

```toml
    [dev-dependencies]
    criterion = "0.1.1"

    [[bench]]
    name = "my_benchmark"
    harness = false
```

Next, define a benchmark by creating a file at `$PROJECT/benches/my_benchmark.rs` with the following contents.

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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Finally, run this benchmark with `cargo bench`. You should see output similar to the following:

```
     Running target/release/deps/example-423eedc43b2b3a93
Benchmarking fib 20
Benchmarking fib 20: Warming up for 3.0000 s
Benchmarking fib 20: Collecting 100 samples in estimated 5.0658 s (188100 iterations)
Benchmarking fib 20: Analyzing
fib 20                  time:   [26.029 us 26.251 us 26.505 us]
Found 11 outliers among 99 measurements (11.11%)
  6 (6.06%) high mild
  5 (5.05%) high severe
slope  [26.029 us 26.505 us] R^2            [0.8745662 0.8728027]
mean   [26.106 us 26.561 us] std. dev.      [808.98 ns 1.4722 us]
median [25.733 us 25.988 us] med. abs. dev. [234.09 ns 544.07 ns]
```

See the [Getting Started](https://japaric.github.io/criterion.rs/book/getting_started.html) guide for more details.

### Goals

The primary goal of Criterion.<span></span>rs is to provide a powerful and statistically rigorous tool for measuring the performance of code, preventing performance regressions and accurately measuring optimizations. Additionally, it should be as programmer-friendly as possible and make it easy to create reliable, useful benchmarks, even for programmers without an advanced background in statistics.

The statistical analysis is mostly solid already; the next few releases will focus mostly on improving ease of use.

### Contributing

First, thank you for contributing.

One great way to contribute to Criterion.<span></span>rs is to use it for your own benchmarking needs and report your experiences, file and comment on issues, etc.

Code or documentation improvements in the form of pull requests are also welcome.

If your issues or pull requests have no response after a few days, feel free to ping me (@bheisler)

For more details, see the [CONTRIBUTING.md file](https://github.com/japaric/criterion.rs/blob/master/CONTRIBUTING.md)

### Maintenance

Criterion.<span></span>rs was originally created by Jorge Aparicio (@japaric) and is currently being maintained by Brook Heisler (@bheisler).

### License

Criterion.<span></span>rs is dual licensed under the Apache 2.0 license and the MIT license.

### Related Projects

- [bencher](https://github.com/bluss/bencher) - A port of the libtest benchmark runner to stable Rust
- [criterion](http://www.serpentine.com/criterion/) - The Haskell microbenchmarking library that inspired Criterion.<span></span>rs
- [cargo-benchcmp](https://github.com/BurntSushi/cargo-benchcmp) - Cargo subcommand to compare the output of two libtest or bencher benchmark runs
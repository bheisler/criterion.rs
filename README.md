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
- Benchmark external programs written in any language.

### Quickstart

Criterion.<span></span>rs currently requires a nightly version of Rust. Additionally, in order to generate plots, you must have [gnuplot](http://www.gnuplot.info/) installed. See the gnuplot website for installation instructions.

To start with Criterion.<span></span>rs, add the following to your `cargo.toml` file:

```toml
    [dev-dependencies]
    criterion = "0.1.0"
```

Next, define a benchmark by creating a file at `$PROJECT/benches/my_benchmark.rs` with the following contents.

```rust
extern crate criterion;

use criterion::Criterion;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

#[test]
fn criterion_benchmark() {
    Criterion::default()
        .bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}
```

Finally, run this benchmark with `cargo bench -- criterion_benchmark --test --nocapture`. You should see output similar to the following:

```
     Running target\release\deps\criterion_example-c6a3683ae7e18b5a.exe

running 1 test
Gnuplot not found, disabling plotting
Benchmarking fib 20
> Warming up for 3.0000 s
> Collecting 100 samples in estimated 5.0726 s
> Found 11 outliers among 99 measurements (11.11%)
  > 2 (2.02%) high mild
  > 9 (9.09%) high severe
> Performing linear regression
  >  slope [26.778 us 27.139 us]
  >    R^2  0.8382863 0.8358049
> Estimating the statistics of the sample
  >   mean [26.913 us 27.481 us]
  > median [26.706 us 26.910 us]
  >    MAD [276.37 ns 423.53 ns]
  >     SD [729.17 ns 2.0625 us]

test criterion_benchmark ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
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

### Maintenance

Criterion.<span></span>rs was originally created by Jorge Aparicio (@japaric) and is currently being maintained by Brook Heisler (@bheisler).

### License

Criterion.<span></span>rs is dual licensed under the Apache 2.0 license and the MIT license.

### Related Projects

- [bencher](https://github.com/bluss/bencher) - A port of the libtest benchmark runner to stable Rust
- [criterion](http://www.serpentine.com/criterion/) - The Haskell microbenchmarking library that inspired Criterion.<span></span>rs
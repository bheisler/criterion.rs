# Comparing Functions

Criterion.rs can automatically benchmark multiple implementations of a function and produce summary graphs to show the differences in performance between them. First, lets create a comparison benchmark.

```rust
#[macro_use]
extern crate criterion;
use criterion::{Criterion, ParameterizedBenchmark}

fn fibonacci_slow(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci_slow(n-1) + fibonacci_slow(n-2),
    }
}

fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut c = 0u64;

    if n == 0 {
        return 0
    }

    for _ in 0..(n+1) {
        c = a + b;
        a = b;
        b = c;
    }
    return b;
}

fn bench_fibs(c: &mut Criterion) {
    c.bench(
        "Fibonacci",
        ParameterizedBenchmark::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)), vec![20u64, 21u64])
            .with_function("Iterative", |b, i| b.iter(|| fibonacci_fast(*i))),
    );
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);
```

These are the same two fibonacci functions from the [Getting Started](./getting_started.html) page. The difference here is that we import the [ParameterizedBenchmark type](http://bheisler.github.io/criterion.rs/criterion/struct.ParameterizedBenchmark.html) as well.

```rust
fn bench_fibs(c: &mut Criterion) {
    c.bench(
        "Fibonacci",
        ParameterizedBenchmark::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)), vec![2u64, 5, 10, 20])
            .with_function("Iterative", |b, i| b.iter(|| fibonacci_fast(*i))),
    );
}
```

Here, we define a `ParameterizedBenchmark` which calls the recursive implementation with several
different inputs. We also add a second benchmark which calls the iterative implementation with the
same inputs. This is then passed to the `Criterion::bench` function, which executes each benchmark
with each input. Criterion will generate a report for each individual benchmark/input pair, as well
as summary reports for each benchmark (across all inputs) and each input (across all benchmarks),
as well as an overall summary of the whole benchmark group.

For benchmarks which do not accept a parameter, there is also the `Benchmark` struct, which is
identical to `ParameterizedBenchmark` except it does not accept parameters.

## Violin Plot

![Violin Plot](./violin_plot.svg)

The [Violin Plot](https://en.wikipedia.org/wiki/Violin_plot) shows the median times and the PDF of
each implementation.

## Line Chart

![Line Chart](./lines.svg)

The line chart shows a comparison of the different functions as the input or input size increases,
which can be enabled with ParameterizedBenchmark.

    

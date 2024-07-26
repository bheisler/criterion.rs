# Getting Started #

This is a quick walkthrough for adding Criterion.rs benchmarks to an existing crate.

I'll assume that we have a crate, `mycrate`, whose `lib.rs` contains the following code:

```rust
#[inline]
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}
```

### Step 1 - Add Dependency to Cargo.toml ###

To enable Criterion.rs benchmarks, add the following to your `Cargo.toml` file:

```toml
[dev-dependencies]
criterion = "0.5.1"

[[bench]]
name = "my_benchmark"
harness = false
```

This adds a development dependency on Criterion.rs, and declares a benchmark called `my_benchmark`
without the standard benchmarking harness. It's important to disable the standard benchmark
harness, because we'll later add our own and we don't want them to conflict.

### Step 2 - Add Benchmark ###

As an example, we'll benchmark our implementation of the Fibonacci function. Create a benchmark
file at `$PROJECT/benches/my_benchmark.rs` with the following contents (see the Details section
below for an explanation of this code):

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use mycrate::fibonacci;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

### Step 3 - Run Benchmark ###

To run this benchmark, use the following command:

`cargo bench`

You should see output similar to this:

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

### Details ###

Let's go back and walk through that benchmark code in more detail.

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use mycrate::fibonacci;
```

First, we declare the criterion crate and import the [Criterion
type](http://bheisler.github.io/criterion.rs/criterion/struct.Criterion.html). Criterion is the
main type for the Criterion.rs library. It provides methods to configure and define groups of
benchmarks. We also import `black_box`, which will be described later.

In addition to this, we declare `mycrate` as an external crate and import our fibonacci function
from it. Cargo compiles benchmarks (or at least, the ones in `/benches`) as if each one was a
separate crate from the main crate. This means that we need to import our library crate as an
external crate, and it means that we can only benchmark public functions.

```rust
fn criterion_benchmark(c: &mut Criterion) {
```

Here we create a function to contain our benchmark code. The name of this function doesn't matter,
but it should be clear and understandable.

```rust
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}
```

This is where the real work happens. The `bench_function` method defines a benchmark with a name
and a closure. The name should be unique among all of the benchmarks for your project. The closure
must accept one argument, a
[Bencher](http://bheisler.github.io/criterion.rs/criterion/struct.Bencher.html). The bencher
performs the benchmark - in this case, it simply calls our `fibonacci` function in a loop. There
are a number of other ways to perform benchmarks, including the option to benchmark with arguments,
and to compare the performance of two functions. See the API documentation for details on all of
the different benchmarking options. Using the `black_box` function stops the compiler from
constant-folding away the whole function and replacing it with a constant.

You may recall that we marked the `fibonacci` function as `#[inline]`. This allows it to be inlined
across different crates. Since the benchmarks are technically a separate crate, that means it can
be inlined into the benchmark, improving performance.

```rust
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Here we invoke the `criterion_group!`
[(link)](http://bheisler.github.io/criterion.rs/criterion/macro.criterion_group.html) macro to
generate a benchmark group called benches, containing the `criterion_benchmark` function defined
earlier. Finally, we invoke the `criterion_main!`
[(link)](http://bheisler.github.io/criterion.rs/criterion/macro.criterion_main.html) macro to
generate a main function which executes the `benches` group. See the API documentation for more
information on these macros.

### Step 4 - Optimize ###

This fibonacci function is quite inefficient. We can do better:

```rust
fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}
```

Running the benchmark now produces output like this:

```
     Running target/release/deps/example-423eedc43b2b3a93
Benchmarking fib 20
Benchmarking fib 20: Warming up for 3.0000 s
Benchmarking fib 20: Collecting 100 samples in estimated 5.0000 s (13548862800 iterations)
Benchmarking fib 20: Analyzing
fib 20                  time:   [353.59 ps 356.19 ps 359.07 ps]
                        change: [-99.999% -99.999% -99.999%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 99 measurements (6.06%)
  4 (4.04%) high mild
  2 (2.02%) high severe
slope  [353.59 ps 359.07 ps] R^2            [0.8734356 0.8722124]
mean   [356.57 ps 362.74 ps] std. dev.      [10.672 ps 20.419 ps]
median [351.57 ps 355.85 ps] med. abs. dev. [4.6479 ps 10.059 ps]
```

As you can see, Criterion is statistically confident that our optimization has made an improvement.
If we introduce a performance regression, Criterion will instead print a message indicating this.

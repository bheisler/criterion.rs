# Getting Started #

### Step 1 - Add Dependency to cargo.toml ###

To enable Criterion.rs benchmarks, add the following to your `cargo.toml` file:

```toml
[dev-dependencies]
criterion = "0.2"

[[bench]]
name = "my_benchmark"
harness = false
```

This adds a development dependency on Criterion.rs, and declares a benchmark called `my_benchmark` without the standard benchmarking harness. It's important to disable the standard benchmark harness, because we'll later add our own and we don't want them to conflict.

### Step 2 - Add Benchmark ###

As an example, we'll benchmark an implementation of the Fibonacci function. Create a benchmark file at `$PROJECT/benches/my_benchmark.rs` with the following contents (see the Details section below for an explanation of this code):

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
#[macro use]
extern crate criterion;

use criterion::Criterion;
```

First, we declare the criterion crate and import the [Criterion type](http://japaric.github.io/criterion.rs/criterion/struct.Criterion.html). Criterion is the main type for the Criterion.rs library. It provides methods to configure and define groups of benchmarks.

```rust
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}
```

Second, we define the function to benchmark. In normal usage, this would be imported from elsewhere in your crate, but for simplicity we'll just define it right here.

```rust
fn criterion_benchmark(c: &mut Criterion) {
```

Here we create a function to contain our benchmark code. The name of the benchmark function doesn't matter, but it should be clear and understandable.

```rust
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}
```

This is where the real work happens. The `bench_function` method defines a benchmark with a name and a closure. The name should be unique among all of the benchmarks for your project. The closure must accept one argument, a [Bencher](http://japaric.github.io/criterion.rs/criterion/struct.Bencher.html). The bencher performs the benchmark - in this case, it simply calls our `fibonacci` function in a loop. There are a number of other benchmark functions, including the option to benchmark with arguments, to benchmark external programs and to compare the performance of two functions. See the API documentation for details on all of the different benchmarking options.

```rust
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Here we invoke the `criterion_group!` [(link)](http://japaric.github.io/criterion.rs/criterion/macro.criterion_group.html) macro to generate a benchmark group called benches, containing the `criterion_benchmark` function defined earlier. Finally, we invoke the `criterion_main!` [(link)](http://japaric.github.io/criterion.rs/criterion/macro.criterion_main.html) macro to generate a main function which executes the `benches` group. See the API documentation for more information on these macros.

### Step 4 - Optimize ###

This fibonacci function is quite inefficient. We can do better:

```rust
fn fibonacci(n: u64) -> u64 {
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

As you can see, Criterion is statistically confident that our optimization has made an improvement. If we introduce a performance regression, Criterion will instead print a message indicating this.

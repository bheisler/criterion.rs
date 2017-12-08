# Getting Started #

Note that Criterion.rs requires a nightly version of Rust.

### Step 1 - Add Dependency to cargo.toml ###

To enable Criterion.rs benchmarks, add the following to your `cargo.toml` file:

```toml
[dev-dependencies]
criterion = "0.1.0"
```

### Step 2 - Add Benchmark ###

As an example, we'll benchmark an implementation of the Fibonacci function. Create a benchmark file at `$PROJECT/benches/my_benchmark.rs` with the following contents (see the Details section below for an explanation of this code):

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

### Step 3 - Run Benchmark ###

To run this benchmark, use the following command:

`cargo bench -- criterion_benchmark --test --nocapture`

You should see output similar to this:

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

### Details ###

Let's go back and walk through that benchmark code in more detail.

```rust
extern crate criterion;

use criterion::Criterion;
```

First, we declare the criterion crate and import the [Criterion type](http://japaric.github.io/criterion.rs/criterion/struct.Criterion.html). Criterion is the main type for the Criterion library. It provides methods to configure and define groups of benchmarks.

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
#[test]
fn criterion_benchmark() {
    Criterion::default()
```

Here we create a test to contain our benchmark code. The name of the benchmark test doesn't matter, but it should be clear and understandable. We start a benchmark by creating an instance of the Criterion struct.

```rust
        .bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}
```

This is where the real work happens. The `bench_function` method defines a benchmark with a name and a closure. THe name should be unique among all of the benchmarks for your project. The closure must accept one argument, a [Bencher](http://japaric.github.io/criterion.rs/criterion/struct.Bencher.html). The bencher performs the benchmark - in this case, it simply calls our `fibonacci` function in a loop. There are a number of other benchmark functions, including the option to benchmark with arguments, to benchmark external programs and to compare the performance of two functions. See the API documentation for details on all of the different benchmarking options.

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
     Running target\release\deps\criterion_example-c6a3683ae7e18b5a.exe

running 1 test
Gnuplot not found, disabling plotting
Benchmarking fib 20
> Warming up for 3.0000 s
> Collecting 100 samples in estimated 5.0000 s
> Found 9 outliers among 99 measurements (9.09%)
  > 5 (5.05%) high mild
  > 4 (4.04%) high severe
> Performing linear regression
  >  slope [428.43 ps 456.05 ps]
  >    R^2  0.2214335 0.2189765
> Estimating the statistics of the sample
  >   mean [431.59 ps 461.16 ps]
  > median [403.16 ps 419.31 ps]
  >    MAD [6.6660 ps 28.954 ps]
  >     SD [53.404 ps 94.558 ps]
fib 20: Comparing with previous sample
> Performing a two-sample t-test
  > H0: Both samples have the same mean
  > p = 0
  > Strong evidence to reject the null hypothesis
> Estimating relative change of statistics
  >   mean [-99.998% -99.998%]
  > median [-99.998% -99.998%]
  > mean has improved by 100.00%
  > median has improved by 100.00%

test criterion_benchmark ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

As you can see, Criterion is statistically confident that our optimization has made an improvement. If we introduce a performance regression, Criterion will instead fail the test by panicking.
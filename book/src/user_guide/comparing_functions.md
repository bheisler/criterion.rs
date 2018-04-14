# Comparing Functions

Criterion.rs can automatically benchmark multiple implementations of a function and produce summary graphs to show the differences in performance between them. First, lets create a comparison benchmark.

```rust
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
```

These are the same two fibonacci functions from the [Getting Started](./getting_started.html) page. The difference here is that we import the [Fun type](http://japaric.github.io/criterion.rs/criterion/struct.Fun.html) as well.

```rust
fn fibonaccis(c: &mut Criterion) {
    let fib_slow = Fun::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)));
    let fib_fast = Fun::new("Iterative", |b, i| b.iter(|| fibonacci_fast(*i)));
```

Here, we create two benchmark functions which simply call our two Fibonacci implementations. Notice that the closure takes two arguments - b is the Bencher as in other examples, and i is the input parameter to be given to the benchmarked function.

```rust
let functions = vec!(fib_slow, fib_fast);

c.bench_functions("Fibonacci", functions, &20);
```

Finally, we construct a Vec of the benchmark functions and run the benchmark. This performs two benchmarks ("Fibonacci/Recursive" and "Fibonacci/Iterative") which individually behave the same as other benchmarks seen earlier. In addition to the usual set of plots generated for each individual benchmark, this will generate a set of summary plots at `.criterion/$BENCHMARK/Summary` highlighting the differences between the functions.

## Violin Plot

![Violin Plot](./user_guide/violin_plot.svg)

The [Violin Plot](https://en.wikipedia.org/wiki/Violin_plot) shows the median times and the PDF of each implementation.

## Line Chart

![Line Chart](./user_guide/lines.svg)

The line chart shows a comparison of the different functions as the input or input size increases, which can be enabled with ParameterizedBenchmark.

```rust
let parameters = vec![5, 10];
let mut benchmark = ParameterizedBenchmark::new("print", |b, i| b.iter(|| print!("{}", i) ), parameters)
.with_function("format", |b, i| b.iter(|| format!("{}", i)));

c.bench("test_bench_param", benchmark);
```
    

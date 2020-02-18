# Benchmarking With Inputs

Criterion.rs can run benchmarks with one or more different input values to investigate how the
performance behavior changes with different inputs.

## Benchmarking With One Input

If you only have one input to your function, you can use a simple interface on the `Criterion` struct
to run that benchmark.

```rust
use std::iter;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;

fn do_something(size: usize) {
    // Do something with the size
}

fn from_elem(c: &mut Criterion) {
    let size: usize = 1024;

    c.bench_with_input(BenchmarkId::new("input_example", size), size, |b, &s| {
        b.iter(|| do_something(s));
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
```

This is convenient in that it automatically passes the input through a `black_box` so that you don't
need to call that directly. It also includes the size in the benchmark description.

## Benchmarking With A Range Of Values

Criterion.rs can compare the performance of a function over a range of inputs using a 
`BenchmarkGroup`.

```rust
use std::iter;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;

fn from_elem(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("from_elem");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
        });
    }
    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
```

In this example, we're benchmarking the time it takes to collect an iterator producing a sequence of
N bytes into a Vec. First, we create a benchmark group, which is a way of telling Criterion.rs that
a set of benchmarks are all related. Criterion.rs will generate extra summary pages for benchmark
groups. Then we simply iterate over a set of desired inputs; we could just as easily unroll this
loop manually, generate inputs of a particular size, etc.

Inside the loop, we call the `throughput` function which informs Criterion.rs that the benchmark
operates on `size` bytes per iteration. Criterion.rs will use this to estimate the number of bytes
per second that our function can process. Next we call `bench_with_input`, providing a unique
benchmark ID (in this case it's just the size, but you could generate custom strings as needed),
passing in the size and a lambda that takes the size and a `Bencher` and performs the actual
measurement.

Finally, we `finish` the benchmark group; this generates the summary pages for that group. It is
recommended to call `finish` explicitly, but if you forget it will be called automatically when the
group is dropped.

![Line Chart](./line.svg)

Here we can see that there is a approximately-linear relationship between the length of an iterator and the time taken to collect it into a Vec.

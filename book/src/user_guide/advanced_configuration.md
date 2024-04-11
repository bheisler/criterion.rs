# Advanced Configuration

Criterion.rs provides a number of configuration options for more-complex use cases. These options are documented here.

## Configuring Sample Count & Other Statistical Settings

Criterion.rs allows the user to adjust certain statistical parameters. The most common way to set
these is using the `BenchmarkGroup` structure - see the documentation for that structure for a list
of which settings are available.

```rust
use criterion::*;

fn my_function() {
    ...
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.1).sample_size(500);
    group.bench_function("my-function", |b| b.iter(|| my_function()));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

It is also possible to change Criterion.rs' default values for these settings, by using the full
form of the `criterion_group` macro:

```rust
use criterion::*;

fn my_function() {
    ...
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    group.bench_function("my-function", |b| b.iter(|| my_function()));
    group.finish();
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(500);
    targets = bench
}
criterion_main!(benches);
```

## Throughput Measurements

When benchmarking some types of code it is useful to measure the throughput as well as the iteration time, either in bytes per second or elements per second. Criterion.rs can estimate the throughput of a benchmark, but it needs to know how many bytes or elements each iteration will process.

Throughput measurements are only supported when using the `BenchmarkGroup` structure; it is not available when using the simpler `bench_function` interface.

To measure throughput, use the `throughput` method on `BenchmarkGroup`, like so:

```rust
use criterion::*;

fn decode(bytes: &[u8]) {
    // Decode the bytes
    ...
}

fn bench(c: &mut Criterion) {
    let bytes : &[u8] = ...;

    let mut group = c.benchmark_group("throughput-example");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("decode", |b| b.iter(|| decode(bytes));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

For parameterized benchmarks, you can simply call the throughput function inside a loop:

```rust
use criterion::*;

type Element = ...;

fn encode(elements: &[Element]) {
    // Encode the elements
    ...
}

fn bench(c: &mut Criterion) {
    let elements_1 : &[u8] = ...;
    let elements_2 : &[u8] = ...;

    let mut group = c.benchmark_group("throughput-example");
    for (i, elements) in [elements_1, elements_2].iter().enumerate() {
        group.throughput(Throughput::Elements(elements.len() as u64));
        group.bench_with_input(format!("Encode {}", i), elements, |b, elems| {
            b.iter(||encode(elems))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

Setting the throughput causes a throughput estimate to appear in the output:

```
alloc                   time:   [5.9846 ms 6.0192 ms 6.0623 ms]
                        thrpt:  [164.95 MiB/s 166.14 MiB/s 167.10 MiB/s]  
```

## Chart Axis Scaling

By default, Criterion.rs generates plots using a linear-scale axis. When using parameterized benchmarks, it is common for the input sizes to scale exponentially in order to cover a wide range of possible inputs. In this situation, it may be easier to read the resulting plots with a logarithmic axis.

As with throughput measurements above, this option is only available when using the `BenchmarkGroup` structure.

```rust
use criterion::*;

fn do_a_thing(x: u64) {
    // Do something
    ...
}

fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("log_scale_example");
    group.plot_config(plot_config);
    
    for i in [1u64, 10u64, 100u64, 1000u64, 10000u64, 100000u64, 1000000u64].iter() {
        group.bench_function(BenchmarkId::from_parameter(i), i, |b, i| b.iter(|| do_a_thing(i)));
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

Currently the axis scaling is the only option that can be set on the 
PlotConfiguration struct. More may be added in the future.

## Sampling Mode

By default, Criterion.rs can scale well to handle benchmarks that execute in picoseconds up to
benchmarks that execute in milliseconds. Benchmarks that take longer will work just fine, but they
tend to take a long time to run. The only way to deal with this was to reduce the sample count.

In Criterion.rs 0.3.3, a new option was added to change the sampling mode to handle long-running
benchmarks. The benchmark author can call `BenchmarkGroup::sampling_mode(SamplingMode)` to change
the sampling mode.

Currently three options are available:
* `SamplingMode::Auto`, which chooses a sampling mode from the other options automatically. This is the default.
* `SamplingMode::Linear`, the original sampling mode intended for faster benchmarks.
* `SamplingMode::Flat`, intended for long-running benchmarks.

The Flat sampling mode does change some of the statistical analysis and the charts that are 
generated. It is not recommended to use Flat sampling except where necessary.

```rust
use criterion::*;
use std::time::Duration;

fn my_function() {
    ::std::thread::sleep(Duration::from_millis(10))
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    group.sampling_mode(SamplingMode::Flat);
    group.bench_function("my-function", |b| b.iter(|| my_function()));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

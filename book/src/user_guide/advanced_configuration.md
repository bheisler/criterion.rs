# Advanced Configuration

Criterion.rs provides a number of configuration options for more-complex use cases. These options are documented here.

## Throughput Measurements

When benchmarking some types of code it is useful to measure the throughput as well as the iteration time, either in bytes per second or elements per second. Criterion.rs can estimate the throughput of a benchmark, but it needs to know how many bytes or elements each iteration will process.

Throughput measurements are only supported with using the `Benchmark` or `ParameterizedBenchmark` structures; it is not available when using the simpler `bench_function` interface.

To measure throughput, use the `throughput` method on `Benchmark`, like so:

```rust
use criterion::*;

fn decode(bytes: &[u8]) {
    // Decode the bytes
    ...
}

fn bench(c: &mut Criterion) {
    let bytes : &[u8] = ...;

    c.bench(
        "throughput-example",
        Benchmark::new(
            "decode",
            |b| b.iter(|| decode(bytes)),
        ).throughput(Throughput::Bytes(bytes.len() as u32)),
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

For parameterized benchmarks, each argument might represent a different number of elements, so the throughput function accepts a lambda instead:

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

    c.bench(
        "throughput-example",
        ParameterizedBenchmark::new(
            "encode",
            |b, elems| b.iter(|| encode(elems)),
            vec![elements_1, elements_2],
        ).throughput(|elems| Throughput::Elements(elems.len() as u32)),
    );
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

As with throughput measurements above, this option is only available when using the `ParameterizedBenchmark` structure.

```rust
use criterion::*;

fn do_a_thing(x: u64) {
    // Do something
    ...
}

fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    c.bench(
        "log_scale_example",
        ParameterizedBenchmark::new(
            "do_thing",
            |b, i| b.iter(|| do_a_thing(i)),
            vec![1u64, 10u64, 100u64, 1000u64, 10000u64, 100000u64, 1000000u64],
        ).plot_config(plot_config),
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

Currently the axis scaling is the only option that can be set on the 
PlotConfiguration struct. More may be added in the future.
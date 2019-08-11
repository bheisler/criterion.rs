## Migrating from 0.2.* to 0.3.*

Criterion.rs took advantage of 0.3.0 being a breaking-change release to make a number of changes
that will require changes to user code.

### `Benchmark`, `ParameterizedBenchmark`, `Criterion::bench_functions`, `Criterion::bench_function_over_inputs`, `Criterion::bench` are deprecated.

In the interest of minimizing disruption, all of these functions still exist and still work. They
are deliberately hidden from the documentation and should not be used in new code. At some point in
the lifecycle of the 0.3.0 series these will be formally deprecated and will start producing
deprecation warnings. They will be removed in 0.4.0.

All of these types and functions have been superseded by the `BenchmarkGroup` type, which is cleaner
to use as well as more powerful and flexible.

### `cargo bench -- --test` is deprecated.

Use `cargo test --benches` instead.

### The format of the `raw.csv` file has changed to accommodate custom measurements.

The `sample_time_nanos` field has been split into `sample_measured_value` and `unit`. For the
default `WallTime` measurement, the `sample_measured_value` is the same as the `sample_time_nanos`
was previously.

### External program benchmarks have been removed.

These were deprecated in version 0.2.6, as they were not used widely enough to justify the extra
maintenance work. It is still possible to benchmark external programs using the `iter_custom`
timing loop, but it does require some extra work. Although it does require extra development effort
on the part of the benchmark author, using `iter_custom` gives more flexibility in how the benchmark
communicates with the external process and also allows benchmarks to work with custom measurements,
which was not possible previously. For an example of benchmarking an external process, see the 
`benches/external_process.rs` benchmark in the Criterion.rs repository.

### Throughput has been expanded to `u64`

Existing benchmarks with u32 Throughputs will need to be changed. Using u64 allows Throughput to
scale up to much larger numbers of bytes/elements.
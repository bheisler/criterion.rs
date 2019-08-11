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
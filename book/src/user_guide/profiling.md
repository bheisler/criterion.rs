# Profiling

When optimizing code, it's often helpful to profile it to help understand why
it produces the measured performance characteristics. Criterion.rs has several
features to assist with profiling benchmarks.

### `--profile-time`

Criterion.rs benchmark executables accept a `--profile-time <num_seconds>` 
argument. If this argument is provided to a run, the benchmark executable will
attempt to iterate the benchmark executable for approximately the given number
of seconds, but will not perform its usual analysis or save any results.
This way, Criterion.rs' analysis code won't appear in the profiling
measurements.

For users of external profilers such as Linux perf, simply run the benchmark
executable(s) under your favorite profiler, passing the profile-time argument.
For users of in-process profilers such as Google's `cpuprofiler`, read on.

### Implementing In-Process Profiling Hooks

For developers who wish to use profiling hooks provided by an existing crate, skip to 
["Enabling In-Process Profiling"](#enabling-in-process-profiling) below.

Since version 0.3.0, Criterion.rs has supported adding hooks to start and stop
an in-process profiler such as [cpuprofiler](https://crates.io/crates/cpuprofiler).
This hook takes the form of a trait, `criterion::profiler::Profiler`.

```rust
pub trait Profiler {
    fn start_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path);
    fn stop_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path);
}
```

These functions will be called before and after each benchmark when running in
`--profile-time` mode, and will not be called otherwise. This makes it easy to
integrate in-process profiling into benchmarks when wanted, without having the
profiling instrumentation affect regular benchmark measurements.

### Enabling In-Process Profiling

Once you (or an external crate) have defined a profiler hook, using it is relatively easy.
You will need to override the `Criterion` struct (which defaults to `ExternalProfiler`) by providing your
own measurement using the `with_profiler` function and overriding the default `Criterion` object
configuration.

```rust
extern crate my_custom_profiler;
use my_custom_profiler::MyCustomProfiler;

fn fibonacci_profiled(criterion: &mut Criterion) {
    // Use the criterion struct as normal here.
}

fn profiled() -> Criterion {
    Criterion::default().with_profiler(MyCustomProfiler)
}

criterion_group! {
    name = benches;
    config = profiled();
    targets = fibonacci_profiled
}
```

The profiler hook will only take effect when running in `--profile-time` mode.
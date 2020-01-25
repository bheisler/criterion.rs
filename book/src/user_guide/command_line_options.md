# Command-Line Options

**Note: If `cargo bench` fails with an error message about an unknown argument, see [the FAQ](../faq.html#cargo-bench-gives-unrecognized-option-errors-for-valid-command-line-options).**

Criterion.rs benchmarks accept a number of custom command-line parameters. This
is a list of the most common options. Run `cargo bench -- -h` to see a full
list.

* To filter benchmarks, use `cargo bench -- <filter>` where `<filter>` is a
regular expression matching the benchmark ID. For example, running 
`cargo bench -- fib_20` would only run benchmarks whose ID contains the string 
`fib_20`, while `cargo bench -- fib_\d+` would also match `fib_300`.
* To print more detailed output, use `cargo bench -- --verbose`
* To disable colored output, use `cargo bench -- --color never`
* To disable plot generation, use `cargo bench -- --noplot`
* To iterate each benchmark for a fixed length of time without saving, analyzing or plotting the results, use `cargo bench -- --profile-time <num_seconds>`. This is useful when profiling the benchmarks. It reduces the amount of unrelated clutter in the profiling results and prevents Criterion.rs' normal dynamic sampling logic from greatly increasing the runtime of the benchmarks.
* To save a baseline, use `cargo bench -- --save-baseline <name>`. To compare against an existing baseline, use `cargo bench -- --baseline <name>`. For more on baselines, see below.
* To test that the benchmarks run successfully without performing the measurement or analysis (eg. in a CI setting), use `cargo test --benches`.
* To override the default plotting backend, use `cargo bench -- --plotting-backend gnuplot` or `cargo bench --plotting-backend plotters`. `gnuplot` is used by default if it is installed.

## Baselines

By default, Criterion.rs will compare the measurements against the previous run (if any). Sometimes it's useful to keep a set of measurements around for several runs. For example, you might want to make multiple changes to the code while comparing against the master branch. For this situation, Criterion.rs supports custom baselines.

* `--save-baseline <name>` will compare against the named baseline, then overwrite it. 
* `--baseline <name>` will compare against the named baseline without overwriting it.
* `--load-baseline <name>` will load the named baseline as the new data set rather than

Using these options, you can manage multiple baseline measurements. For instance, if you want to compare against a static reference point such as the master branch, you might run:

```sh
git checkout master
cargo bench -- --save-baseline master
git checkout feature
cargo bench -- --save-baseline feature
git checkout optimizations

# Some optimization work here

# Measure again
cargo bench
# Now compare against the stored baselines without overwriting it or re-running the measurements
cargo bench -- --load-baseline new --baseline master
cargo bench -- --load-baseline new --baseline feature
```

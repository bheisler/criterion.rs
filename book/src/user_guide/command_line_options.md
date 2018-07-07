# Command-Line Options

Criterion.rs benchmarks accept a number of custom command-line parameters. This
is a list of the most common options. Run `cargo bench -- -h` to see a full
list.

* To filter benchmarks, use `cargo bench -- <filter>` where `<filter>` is a
substring of the benchmark ID. For example, running `cargo bench -- fib_20`
would only run benchmarks whose ID contains the string `fib_20`
* To print more detailed output, use `cargo bench -- --verbose`
* To disable colored output, use `cargo bench -- --color never`
* To disable plot generation, use `cargo bench -- --noplot`
* To only run the measurements, without saving, analyzing or plotting the results, use `cargo bench -- --measure-only`. This is useful when profiling the benchmarks, to reduce the amount of unrelated clutter in the profiling results.
* To save a baseline, use `cargo bench -- --save-baseline <name>`. To compare against an existing baseline, use `cargo bench -- --baseline <name>`. For more on baselines, see below.
* To test that the benchmarks run successfully without performing the measurement or analysis (eg. in a CI setting), use `cargo bench -- --test`.

## Baselines

By default, Criterion.rs will compare the measurements against the previous run (if any). Sometimes it's useful to keep a set of measurements around for several runs. For example, you might want to make multiple changes to the code while comparing against the master branch. For this situation, Criterion.rs supports custom baselines.

* `--save-baseline <name>` will compare against the named baseline, then overwrite it. 
* `--baseline <name>` will compare against the named baseline without overwriting it.

Using these options, you can manage multiple baseline measurements. For instance, if you want to compare against a static reference point such as the master branch, you might run:

```sh
git checkout master
cargo bench -- --save-baseline master
git checkout optimizations
cargo bench -- --baseline master

# Some optimization work here

# Measure again and compare against the stored baseline without overwriting it
cargo bench -- --baseline master
```

## Known Limitations

There are currently a number of limitations to the use of Criterion.rs relative to the standard benchmark harness.

First, it is necessary for Criterion.rs to provide its own `main` function using the `criterion_main` macro.
This results in several limitations:

* It is not possible to include benchmarks in code in the `src/` directory as one might with the regular
  benchmark harness. 
* It is not possible to benchmark non-`pub` functions. External benchmarks, including those using Criterion.rs,
  are compiled as a separate crate, and non-`pub` functions are not visible to the benchmarks.
* It is not possible to benchmark functions in binary crates. Binary crates cannot be dependencies of other
  crates, and that includes external tests and benchmarks ([see here](https://github.com/rust-lang/cargo/issues/4316) for more details)

Criterion.rs cannot currently solve these issues. An [experimental RFC](https://github.com/rust-lang/rust/issues/50297) is being implemented to enable custom test and benchmarking frameworks.

Second, Criterion.rs provides a stable-compatible replacement for the `black_box` function provided by the standard test crate. This replacement is not as reliable as the official one, and it may allow dead-code-elimination to affect the benchmarks in some circumstances. If you're using a Nightly build of Rust, you can add the `real_blackbox` feature to your dependency on Criterion.rs to use the standard `black_box` function instead.

Example:

```toml
criterion = { version = '...', features=['real_blackbox'] }
```
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
* It is not possible to benchmark functions in crates that do not provide an `rlib`.

Criterion.rs cannot currently solve these issues. An [experimental RFC](https://github.com/rust-lang/rust/issues/50297) is being implemented to enable custom test and benchmarking frameworks.

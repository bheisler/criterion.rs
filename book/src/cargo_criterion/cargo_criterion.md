# cargo-criterion

cargo-criterion is an experimental Cargo extension which can act as a replacement for `cargo bench`. The long-term goal for cargo-criterion is to handle all of the statistical analysis and report generation in a single tool. Then, the code for that can be removed from Criterion.rs (or made optional), reducing benchmark compilation and linking time. Since it manages the whole lifecycle of a benchmark run, `cargo-criterion` is also in a good position to provide features that would be difficult to implement in Criterion.rs itself.

Currently, `cargo-criterion` provides most of the same features as running Criterion.rs benchmarks in `cargo bench`, but with some differences:
* `cargo-criterion` does not currently support baselines
* `cargo-criterion` is more configurable than Criterion.rs
* `cargo-criterion` supports machine-readable output using `--message-format=json`

`cargo-criterion` is stable, and you can install it with the following command:

`cargo install cargo-criterion`

Once installed, you can run your benchmarks with:

`cargo criterion`

If you encounter any issues or have any suggestions for future features, please raise an issue at [the GitHub repository](https://github.com/bheisler/cargo-criterion).

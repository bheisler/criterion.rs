# cargo-criterion

cargo-criterion is an experimental Cargo extension which can act as a replacement for `cargo bench`. The long-term goal for cargo-criterion is to handle all of the statistical analysis and report generation in a single tool. Then, the code for that can be removed from Criterion.rs (or made optional), reducing benchmark compilation and linking time. Since it manages the whole lifecycle of a benchmark run, `cargo-criterion` is also in a good position to provide features that would be difficult to implement in Criterion.rs itself.

At time of writing, `cargo-criterion` provides roughly the same features as running Criterion.rs benchmarks in `cargo bench`, but additional features such as machine-readable output and historical performance charts are planned.

`cargo-criterion` is still under active development, but if you would like to try it out, you can install it with the following command:

`cargo install --version=1.0.0-alpha2 cargo-criterion`

Once installed, you can run your benchmarks with:

`cargo criterion`

If you encounter any issues or have any suggestions for future features, please raise an issue at [the GitHub repository](https://github.com/bheisler/cargo-criterion).
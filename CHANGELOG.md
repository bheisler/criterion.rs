# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.11] - 2019-04-08
### Added
- Enabled automatic text-coloring on Windows.

### Fixed
- Fixed panic caused by outdated files after benchmark names or types were changed.
- Reduced timing overhead of `Criterion::iter_batched/iter_batched_ref`.

## [0.2.10] - 2019-02-09
### Added
- Added `iter_batched/iter_batched_ref` timing loops, which allow for setup (like 
  `iter_with_setup/iter_with_large_setup`) and exclude drop (like `iter_with_large_drop`) but
  measure the runtime more accurately, use less memory and are more flexible.

### Deprecated
- `iter_with_setup/iter_with_large_setup` are now deprecated in favor of `iter_batched`.

## [0.2.9] - 2019-01-24
### Changed
- Criterion.rs no longer depends on the default features of the `rand-core` crate. This fixes some
  downstream crates which use `rand` in a `no_std` context.

## [0.2.8] - 2019-01-20
### Changed
- Criterion.rs now uses `rayon` internally instead of manual `unsafe` code built with thread-scoped.
- Replaced handlebars templates with [TinyTemplate](https://github.com/bheisler/TinyTemplate)
- Merged `criterion-stats` crate into `criterion` crate. `criterion-stats` will no longer receive
  updates.
- Replaced or removed various other dependencies to reduce the size of Criterion.rs' dependency 
  tree.

## [0.2.7] - 2018-12-29

### Fixed
- Fixed version numbers to prevent incompatibilities between `criterion` and `criterion-stats`
  crates.

## [0.2.6] - 2018-12-27 - Yanked
### Added
- Added `--list` command line option, which lists the benchmarks but does not run them, to match
  `cargo test -- --list`.
- Added README/CONTRIBUTING/LICENSE files to sub-crates.
- Displays change in throughput in the command-line and HTML output as well as change in iteration 
  time.
- Benchmarks with multiple functions and multiple values will now generate a per-value summary
  report file in addition to the existing per-function one.
- Added a `--profile-time` command-line argument which disables reporting and analysis and instead
  simply iterates each benchmark for approximately the given number of seconds. This supersedes the
  (now-deprecated) `--measure-only` argument.

### Fixed
- Functions passed to `Bencher::iter_with_large_setup` can now return output. This is necessary to 
  prevent the compiler from optimizing away the benchmark. This is technically a breaking change - 
  that function requires a new type parameter. It's so unlikely to break existing code that I
  decided not to delay this for a breaking-change release.
- Reduced measurement overhead for the `iter_with_large_setup` and `iter_with_drop` methods.
- `criterion_group` and `criterion_main` macros no longer require the `Criterion` struct to be
  explicitly imported.
- Don't panic when `gnuplot --version` fails.
- Criterion.rs macros no longer require user to `use criterion::Criterion;`
- Criterion.rs no longer initializes a logger, meaning that it will no longer conflict with user
  code which does.
- Criterion.rs no longer fails to parse gnuplot version numbers like 
  `gnuplot 5.2 patchlevel 5a (Gentoo revision r0)`
- Criterion.rs no longer prints an error message that gnuplot couldn't be found when chart 
  generation is disabled (either by `Criterion::without_plots`, `--noplot` or disabling the 
  HTML reports feature)
- Benchmark names are now automatically truncated to 100 characters and a number may be added to
  make them unique. This fixes a problem where gnuplot would crash if the title was extremely long,
  and also improves the general usability of Criterion.rs.

### Changed
- Changed timing model of `iter_with_large_setup` to exclude time spent dropping values returned
  by the routine. Time measurements taken with 0.2.6 using these methods may differ from those taken
  with 0.2.5.
- Benchmarks with multiple functions and multiple values will now appear as a table rather than a
  tree in the benchmark index. This is to accommodate the new per-value summary reports.

### Deprecated
- Deprecated the `--measure-only` command-line-argument in favor of `--profile-time`. This will be
  removed in 0.3.0.
- External-program benchmarks are now deprecated. They will be removed in 0.3.0.
- The `html_reports` cargo feature is now deprecated. This feature will become non-optional in 0.3.0.
- Sample sizes less than 10 are deprecated and will be disallowed in 0.3.0.
- This is not an exhaustive list - the full scope of changes in 0.3.0 is not yet determined. There
  may be breaking changes that are not listed here.

## [0.2.5] - 2018-08-27
### Fixed
- Fixed links from generated report files to documentation.
- Fixed formatting for very large percentage changes (>1000%)
- Sorted the benchmarks in the index report by name
- Fixed case where benchmark ID with special characters would cause Criterion.rs to open the wrong 
  file and log an error message.
- Fixed case where running `cargo clean; cargo bench -- <filter>` would cause Criterion.rs to log
  an error message.
- Fixed a GNUplot error message when sample size is very small.
- Fixed several cases where Criterion.rs would generate invalid path names.
- Fixed a bug where Criterion.rs would print an error if run with a filter that allowed no benchmarks and a clean target directory.
- Fixed bug where some benchmarks didn't appear in the benchmark index report.
- Criterion.rs now honors the `CARGO_TARGET_DIR` environment variable.

### Added
- Criterion.rs will generate a chart showing the effects of changes in input (or input size) for all
  benchmarks with numeric inputs or throughput, not just for those which compare multiple functions.

## [0.2.4] 2018-07-08
### Added
- Added a pair of flags, `--save-baseline` and `--baseline`, which change
  how benchmark results are stored and compared. This is useful for
  working against a fixed baseline(eg. comparing progress on an
  optimization feature branch to the commit it forked from).
  Default behavior of Criterion.rs is now `--save-baseline base`
  which emulates the previous, user facing behavior.
  - `--save-baseline` saves the benchmark results under the provided name.
  - `--baseline` compares the results to a saved baseline.
    If the baseline does not exist for a benchmark, an error is given.
- Added user-guide documentation for baselines, throughput measurements and
  plot configuration.
- Added a flag, `--test`, which causes Criterion to execute the benchmarks once
  without measuring or reporting the results. This is useful for checking that the
  benchmarks run successfully in a CI setting.
- Added a `raw.csv` file to the output which contains a stable, machine-readable
  representation of the measurements taken by benchmarks. This enables users to
  perform their own analysis or keep historical information without depending on
  private implementation details.

### Fixed
- The `sample_size` method on the `Criterion`, `Benchmark` and 
  `ParameterizedBenchmark` structs has been changed to panic if the sample size
  is less than 2. Other parts of the code require this and will panic if the
  sample size is 1, so this is not considered to be a breaking change.
- API documentation has been updated to show more-complete examples.
- Certain characters will now be replaced with underscores when creating benchmark
  directory paths, to avoid generating invalid or unexpected paths.

## [0.2.3] - 2018-04-14
### Fixed
- Criterion.rs will now panic with a clear error message if the user attempts to run
  a benchmark which doesn't call the `Bencher::iter` function or a related function,
  rather than failing in an uncontrolled manner later.
- Fixed broken links in some more summary reports.

### Added
- Added a `--measure-only` argument which causes the benchmark executable to run the
  warmup and measurement and then move on to the next benchmark without analyzing or
  saving data. This is useful to prevent Criterion.rs' analysis code from appearing
  in profile data when profiling benchmarks.
- Added an index report file at "target/criterion/report/index.html" which links to
  the other reports for easy navigation.

## [0.2.2] - 2018-03-25
### Fixed
- Fixed broken links in some summary reports.
- Work around apparent rustc bug in >= 1.24.0.

## [0.2.1] - 2018-02-24
### Added
- HTML reports are now a default Cargo feature. If you wish to disable HTML reports,
  disable Criterion.rs' default features. Doing so will allow compatibility with
  older Rust versions such as 1.20. If you wish to continue using HTML reports, you
  don't need to do anything.
- Added a summary report for benchmarks that compare multiple functions or different
  inputs.

### Changed
- The plots and HTML reports are now generated in a `report` folder.

### Fixed
- Underscores in benchmark names will no longer cause subscripted characters to
  appear in generated plots.

## [0.2.0] - 2018-02-05
### Added
- Added `Criterion.bench` function, which accepts either a `Benchmark` or
  `ParameterizedBenchmark`. These new structures allow for custom per-benchmark
  configuration as well as more complex benchmark grouping (eg. comparing a Rust
  function against an external program over a range of inputs) which was not
  possible previously.
- Criterion.rs can now report the throughput of the benchmarked code in units of
  bytes or elements per second. See the `Benchmark.throughput` and
  `ParameterizedBenchmark.throughput` functions for further details.
- Criterion.rs now generates a basic HTML report for each benchmark.
- Added `--noplot` command line option to disable plot generation.

### Changed
- The builder methods on the Criterion struct now take and return self by value
  for easier chaining. Functions which configure a Criterion structure will need
  to be updated accordingly, or will need to be changed to work with the
  `Benchmark` or `ParameterizedBenchmark` types to do per-benchmark configuration
  instead.
- The closures taken by `Criterion.bench_*` must now have a `'static` lifetime.
  This means that you may need to change your closures from `|bencher| {...}`
  to `move |bencher| {...}`.
- `Criterion.bench_functions` now takes `I` as an input parameter, not `&I`.
- Input values must now implement `Debug` rather than `Display`.
- The generated plots are stored in `target/criterion` rather than `.criterion`.

### Removed
- The hidden `criterion::ConfidenceInterval` and`criterion::Estimate` types are
  no longer publicly accessible.
- The `Criterion.summarize` function has been removed.

### Fixed
- Fixed the relative mean and median reports.
- Fixed panic while summarizing benchmarks.

## [0.1.2] - 2018-01-12
### Changed
- Criterion.rs is now stable-compatible!
- Criterion.rs now includes its own stable-compatible `black_box` function.
  Some benchmarks may now be affected by dead-code-elimination where they
  previously weren't and may have to be updated.
- Criterion.rs now uses `serde` to save results. Existing results files will
  be automatically removed when benchmarks are run.
- Redesigned the command-line output to highlight the important information
  and reduce noise.

### Added
- Running benchmarks with the variable "CRITERION_DEBUG" in the environment will
  cause Criterion.rs to generate extra debug output and save the gnuplot scripts
  alongside the generated plots.

### Fixed
- Don't panic on IO errors or gnuplot failures
- Fix generation of invalid gnuplot scripts when benchmarking over inputs and inputs include values <= 0.
- Bug where benchmarks would run one sample fewer than was configured.

### Removed
- Generated plots will no longer use log-scale.

## [0.1.1] - 2017-12-12
### Added
- A changelog file.
- Added a chapter to the book on how Criterion.rs collects and analyzes data.
- Added macro rules to generate a test harness for use with `cargo bench`.
  Benchmarks defined without these macros should continue to work.
- New contribution guidelines
- Criterion.rs can selectively run benchmarks. See the Command-line page for
more details

## 0.1.0 - 2017-12-02
### Added
- Initial release on Crates.io.


[Unreleased]: https://github.com/bheisler/criterion.rs/compare/0.2.11...HEAD
[0.1.1]: https://github.com/bheisler/criterion.rs/compare/0.1.0...0.1.1
[0.1.2]: https://github.com/bheisler/criterion.rs/compare/0.1.1...0.1.2
[0.2.0]: https://github.com/bheisler/criterion.rs/compare/0.1.2...0.2.0
[0.2.1]: https://github.com/bheisler/criterion.rs/compare/0.2.0...0.2.1
[0.2.2]: https://github.com/bheisler/criterion.rs/compare/0.2.1...0.2.2
[0.2.3]: https://github.com/bheisler/criterion.rs/compare/0.2.2...0.2.3
[0.2.4]: https://github.com/bheisler/criterion.rs/compare/0.2.3...0.2.4
[0.2.5]: https://github.com/bheisler/criterion.rs/compare/0.2.4...0.2.5
[0.2.6]: https://github.com/bheisler/criterion.rs/compare/0.2.5...0.2.6
[0.2.7]: https://github.com/bheisler/criterion.rs/compare/0.2.6...0.2.7
[0.2.8]: https://github.com/bheisler/criterion.rs/compare/0.2.7...0.2.8
[0.2.9]: https://github.com/bheisler/criterion.rs/compare/0.2.8...0.2.9
[0.2.10]: https://github.com/bheisler/criterion.rs/compare/0.2.9...0.2.10
[0.2.11]: https://github.com/bheisler/criterion.rs/compare/0.2.10...0.2.11

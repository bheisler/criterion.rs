## Frequently Asked Questions

### How Should I Run Criterion.rs Benchmarks In A CI Pipeline?

Criterion.rs benchmarks can be run as part of a CI pipeline just as they
normally would on the command line - simply run `cargo bench`.

To compare the master branch to a pull request, you could run the benchmarks on
the master branch to set a baseline, then run them again with the pull request
branch. An example script for Travis-CI might be:

```bash
#!/usr/bin/env bash

if [ "${TRAVIS_PULL_REQUEST_BRANCH:-$TRAVIS_BRANCH}" != "master" ] && [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
    REMOTE_URL="$(git config --get remote.origin.url)";
    cd ${TRAVIS_BUILD_DIR}/.. && \
    git clone ${REMOTE_URL} "${TRAVIS_REPO_SLUG}-bench" && \
    cd  "${TRAVIS_REPO_SLUG}-bench" && \
    # Bench master
    git checkout master && \
    cargo bench && \
    # Bench pull request
    git checkout ${TRAVIS_COMMIT} && \
    cargo bench;
fi
```

(Thanks to [BeachApe](https://beachape.com/blog/2016/11/02/rust-performance-testing-on-travis-ci/) for the script on which this is based.)

Note that cloud CI providers like Travis-CI and Appveyor introduce a great deal
of noise into the benchmarking process. For example, unpredictable load on the
physical hosts of their build VM's. Benchmarks measured on such services tend
to be unreliable, so you should be skeptical of the results. In particular,
benchmarks that detect performance regressions should not cause the build to
fail, and apparent performance regressions should be verified manually before
rejecting a pull request.

### `cargo bench` Gives "Unrecognized Option" Errors for Valid Command-line Options

By default, Cargo implicitly adds a `libtest` benchmark harness to your crate when benchmarking, to
handle any `#[bench]` functions, even if you have none. It compiles and runs this executable first,
before any of the other benchmarks. Normally, this is fine - it detects that there are no `libtest`
benchmarks to execute and exits, allowing Cargo to move on to the real benchmarks. Unfortunately,
it checks the command-line arguments first, and panics when it finds one it doesn't understand.
This causes Cargo to stop benchmarking early, and it never executes the Criterion.rs benchmarks.

This will occur when running `cargo bench` with any argument that Criterion.rs supports but `libtest`
does not. For example, `--verbose` and `--save-baseline` will cause this issue, while `--help` will
not. There are two ways to work around this at present:

You could run only your Criterion benchmark, like so:

`cargo bench --bench my_benchmark -- --verbose`

Note that `my_benchmark` here corresponds to the name of your benchmark in your
`Cargo.toml` file.

Another option is to disable benchmarks for your lib or app crate. For example,
for library crates, you could add this to your `Cargo.toml` file:

```toml
[lib]
bench = false
```

If your crate produces one or more binaries as well as a library, you may need to add additional
records to `Cargo.toml` like this:

```toml
[[bin]]
name = "my-binary"
path = "src/bin/my-binary.rs"
bench = false
```

This is because Cargo automatically discovers some kinds of binaries and it will enable the default
benchmark harness for these as well.

Of course, this only works if you define all of your benchmarks in the
`benches` directory.

See [Rust Issue #47241](https://github.com/rust-lang/rust/issues/47241) for
more details.

### How Should I Benchmark Small Functions?

Exactly the same way as you would benchmark any other function.

It is sometimes suggested that benchmarks of small (nanosecond-scale) functions should iterate the
function to be benchmarked many times internally to reduce the impact of measurement overhead.
This is _not_ required with Criterion.rs, and it is not recommended.

To see this, consider the following benchmark:

```rust
fn compare_small(c: &mut Criterion) {
    use criterion::black_box;

    let mut group = c.benchmark_group("small");
    group.bench_with_input("unlooped", 10, |b, i| b.iter(|| i + 10));
    group.bench_with_input("looped", 10, |b, i| b.iter(|| {
        for _ in 0..10000 {
            black_box(i + 10);
        }
    }));
    group.finish();
}
```

This benchmark simply adds two numbers - just about the smallest function that could be performed.
On my computer, this produces the following output:

```
small/unlooped          time:   [270.00 ps 270.78 ps 271.56 ps]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
small/looped            time:   [2.7051 us 2.7142 us 2.7238 us]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

2.714 microseconds/10000 gives 271.4 picoseconds, or pretty much the same result. Interestingly,
this is slightly more than one cycle of my 4th-gen Core i7's maximum clock frequency of 4.4 GHz,
which shows how good the pipelining is on modern CPUs. Regardless, Criterion.rs is able to
accurately measure functions all the way down to single instructions. See the [Analysis
Process](./analysis.md) page for more details on how Criterion.rs performs its measurements, or see
the [Timing Loops](./user_guide/timing_loops.md) page for details on choosing a timing loop to minimize
measurement overhead.

### When Should I Use `criterion::black_box`?

`black_box` is a function which prevents certain compiler optimizations. Benchmarks are often
slightly artificial in nature and the compiler can take advantage of that to generate faster code
when compiling the benchmarks than it would in real usage. In particular, it is common for
benchmarked functions to be called with constant parameters, and in some cases rustc can
evaluate the function entirely at compile time and replace the function call with a constant.
This can produce unnaturally fast benchmarks that don't represent how some code would perform
when called normally. Therefore, it's useful to black-box the constant input to prevent this
optimization.

However, you might have a function which you expect to be called with one or more constant
parameters. In this case, you might want to write your benchmark to represent that scenario instead,
and allow the compiler to optimize the constant parameters.

For the most part, Criterion.rs handles this for you - if you use parameterized benchmarks, the
parameters are automatically black-boxed by Criterion.rs so you don't need to do anything. If you're
writing an un-parameterized benchmark of a function that takes an argument, however, this may be
worth considering.

### Cargo Prints a Warning About Explicit [[bench]] Sections in Cargo.toml

Currently, Cargo treats any `*.rs` file in the `benches` directory as a
benchmark, unless there are one or more `[[bench]]` sections in the
`Cargo.toml` file. In that case, the auto-discovery is disabled
entirely.

In Rust 2018 edition, Cargo will be changed so that `[[bench]]` no longer
disables the auto-discovery. If your `benches` directory contains source files
that are not benchmarks, this could break your build when you update, as Cargo
will attempt to compile them as benchmarks and fail.

There are two ways to prevent this breakage from happening. You can explicitly
turn off the autodiscovery like so:

```toml
[[package]]
autobenches = false
```

The other option is to move those non-benchmark files to a subdirectory (eg.
`benches/benchmark_code`) where they will no longer be detected as benchmarks.
I would recommend the latter option.

Note that a file which contains a `criterion_main!` is a valid benchmark and can
safely stay where it is.

### I made a trivial change to my source and Criterion.rs reports a large change in performance. Why?

Don't worry, Criterion.rs isn't broken and you (probably) didn't do anything wrong. The most common
reason for this is that the optimizer just happened to optimize your function differently after the
change.

Optimizing compiler backends such as LLVM (which is used by `rustc`) are often complex beasts full
of hand-rolled pattern matching code that detects when a particular optimization is possible and
tries to guess whether it would make the code faster. Unfortunately, despite all of the engineering
work that goes into these compilers, it's pretty common for apparently-trivial changes to the source
like changing the order of lines to be enough to cause these optimizers to act differently. On top of
this, apparently-small changes like changing the type of a variable or calling a slightly different
function (such as `unwrap` vs `expect`) actually have much larger impacts under the hood than the
slight different in source text might suggest.

If you want to learn more about this (and some proposals for improving this situation in the
future), I like [this paper](https://blog.regehr.org/archives/1619) by Regehr et al.

On a similar subject, it's important to remember that a benchmark is only ever an estimate
of the true performance of your function. If the optimizer can have significant effects on
performance in an artificial environment like a benchmark, what about when your function is inlined
into a variety of different calling contexts? The optimizer will almost certainly make different
decisions for each caller. One hopes that each specialized version will be faster, but that can't
be guaranteed. In a world of optimizing compilers, the "true performance" of a function is a fuzzy
thing indeed.

If you're still sure that Criterion.rs is doing something wrong, file an issue describing the
problem.

### I made _no_ change to my source and Criterion.rs reports a large change in performance. Why?

Typically this happens because the benchmark environments aren't quite the same. There are a lot of
factors that can influence benchmarks. Other processes might be using the CPU or memory.
Battery-powered devices often have power-saving modes that clock down the CPU (and these sometimes
appear in desktops as well). If your benchmarks are run inside a VM, there might be other VMs on the
same physical machine competing for resources.

However, sometimes this happens even with no change. It's important to remember that Criterion.rs
detects regressions and improvements statistically. There is always a chance that you randomly
get unusually fast or slow samples, enough that Criterion.rs detects it as a change even though no
change has occurred. In very large benchmark suites you might expect to see several of these
spurious detections each time you run the benchmarks.

Unfortunately, this is a fundamental trade-off in statistics. In order to decrease the rate of false
detections, you must also decrease the sensitivity to small changes. Conversely, to increase the
sensitivity to small changes, you must also increase the chance of false detections. Criterion.rs
has default settings that strike a generally-good balance between the two, but you can adjust the
settings to suit your needs.

### When I run benchmark executables directly (without using Cargo) they just print "Success". Why?

When Cargo runs benchmarks, it passes the `--bench` or `--test` command-line arguments to the
benchmark executables. Criterion.rs looks for these arguments and tries to either run benchmarks or
run in test mode. In particular, when you run `cargo test --benches` (run tests, including testing
benchmarks) Cargo does not pass either of these arguments. This is perhaps strange, since `cargo
bench --test` passes both `--bench` and `--test`. In any case, Criterion.rs benchmarks run in test
mode when `--bench` is not present, or when `--bench` and `--test` are both present.

### My benchmark fails to compile with the error "use of undeclared type or module `<my_crate>`

First, check the [Getting Started](https://bheisler.github.io/criterion.rs/book/getting_started.html) 
guide and ensure that the `[[bench]]` section of your Cargo.toml is set up correctly. If it's
correct, read on.

This can be caused by two different things.

Most commonly, this problem happens when trying to benchmark a binary (as opposed to library) crate.
Criterion.rs cannot be used to benchmark binary crates (see the 
[Known Limitations](https://bheisler.github.io/criterion.rs/book/user_guide/known_limitations.html)
page for more details on why). The usual workaround is to structure your application as a library
crate that implements most of the functionality of the application and a binary crate which acts
as a thin wrapper around the library crate to provide things like a CLI. Then, you can create
Criterion.rs benchmarks that depend on the library crate.

Less often, the problem is that the library crate is configured to compile as a `cdylib`. In order
to benchmark your crate with Criterion.rs, you will need to set your Cargo.toml to enable generating
an `rlib` as well.
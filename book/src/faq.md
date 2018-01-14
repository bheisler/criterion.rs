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

### `cargo bench -- --verbose` Panics

This occurs because the `libtest` benchmark harness implicitly added to your
crate is executing before the Criterion.rs benchmarks, and it panics when
presented with a command-line argument it doesn't expect. There are two ways to
work around this at present:

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

Of course, this only works if you define all of your benchmarks in the
`benches` directory.

See [Rust Issue #47241](https://github.com/rust-lang/rust/issues/47241) for
more details.
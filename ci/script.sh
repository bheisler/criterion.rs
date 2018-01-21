set -ex

if [ "$CLIPPY" = "yes" ]; then
      cargo clippy --all -- -D warnings
elif [ "$DOCS" = "yes" ]; then
    cargo clean
    cargo doc --all --no-deps
    cd book
    mdbook build
    cd ..
    cp -r book/book/ target/doc/book/
    travis-cargo doc-upload || true
elif [ "$COVERAGE" = "yes" ]; then
    cargo tarpaulin --all --no-count --ciserver travis-ci --coveralls $TRAVIS_JOB_ID
elif [ "$BENCHMARK" = "yes" ]; then
    cargo bench --all
else
    cargo build
    cargo test --all
    cargo build --benches --all
fi

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
    cargo tarpaulin --all --ciserver travis-ci --coveralls $TRAVIS_JOB_ID
else
    cargo build --release
    cargo test --all --release
    cargo build --benches --all --release
    cargo bench
fi

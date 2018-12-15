set -ex

if [ "$HTML_REPORTS" = "no" ]; then
    BUILD_ARGS="--no-default-features"
else
    BUILD_ARGS=""
fi

if [ "$CLIPPY" = "yes" ]; then
      cargo clippy --all -- -D warnings
elif [ "$DOCS" = "yes" ]; then
    cargo clean
    cargo doc --all --no-deps $BUILD_ARGS
    cd book
    mdbook build
    cd ..
    cp -r book/book/ target/doc/book/
    travis-cargo doc-upload || true
elif [ "$RUSTFMT" = "yes" ]; then
    cargo fmt --all -- --check
elif [ "$MINIMAL_VERSIONS" = "yes" ]; then
    rm Cargo.lock || true
    cargo build -Z minimal-versions
else
    cargo build $BUILD_ARGS --release

    # TODO: Remove this hack once we no longer have to support 1.23 and 1.20
    if [ "$TRAVIS_RUST_VERSION" = "stable" ]; then
        cargo test $BUILD_ARGS --all --release
    else
        cargo test $BUILD_ARGS --all --release --tests
    fi

    cargo bench $BUILD_ARGS --all -- --test
    cd bencher_compat
    export CARGO_TARGET_DIR="../target"
    cargo bench $BUILD_ARGS -- --test
fi

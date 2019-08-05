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
elif [ "$RUSTFMT" = "yes" ]; then
    cargo fmt --all -- --check
elif [ "$MINIMAL_VERSIONS" = "yes" ]; then
    rm Cargo.lock || true
    cargo build -Z minimal-versions
else
    cargo build $BUILD_ARGS --release

    # TODO: Remove this hack once we no longer have to support 1.23 and 1.20
    if [ "$TRAVIS_RUST_VERSION" = "stable" ]; then
        cargo test --all --release
    else
        cargo test --all --release --tests
    fi

    cargo bench --all -- --test
    
    cd bencher_compat
    export CARGO_TARGET_DIR="../target"
    cargo bench -- --test
    cd ..

    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
        cd macro
        export CARGO_TARGET_DIR="../target"
        cargo bench -- --test
        cd ..
    fi
fi

set -ex

if [ "$CLIPPY" = "yes" ]; then
      cargo clippy --all -- -D warnings
elif [ "$DOCS" = "yes" ]; then
    cargo clean
    cargo doc --all --no-deps
    cd book
    mdbook build
    cd ..
    cp -r book/book/html/ target/doc/book/
    travis-cargo doc-upload || true
elif [ "$RUSTFMT" = "yes" ]; then
    cargo fmt --all -- --check
elif [ "$MINIMAL_VERSIONS" = "yes" ]; then
    rm Cargo.lock || true
    cargo build -Z minimal-versions
else
    export RUSTFLAGS="-D warnings"

    cargo build $BUILD_ARGS --release

    cargo test --all --release
    cargo test --benches
    
    cd bencher_compat
    export CARGO_TARGET_DIR="../target"
    cargo test --benches
    cd ..

    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
        cd macro
        export CARGO_TARGET_DIR="../target"
        cargo test --benches
        cd ..
    fi
fi

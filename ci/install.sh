set -ex

if [ "$CLIPPY" = "yes" ]; then
  cargo install clippy --force --vers $CLIPPY_VERSION
fi

if [ "$DOCS" = "yes" ]; then
    cargo install mdbook --force
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew update
    brew install gnuplot
fi

if [ "$COVERAGE" = "yes" ]; then
    RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin --force
fi

if [ "$RUSTFMT" = "yes" ]; then
    rustup component add rustfmt-preview
fi

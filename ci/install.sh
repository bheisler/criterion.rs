set -ex

if [ "$CLIPPY" = "yes" ]; then
    rustup component add clippy-preview
fi

if [ "$DOCS" = "yes" ]; then
    cargo install mdbook --no-default-features --force
    cargo install mdbook-linkcheck --force
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew install gnuplot
fi

if [ "$RUSTFMT" = "yes" ]; then
    rustup component add rustfmt-preview
fi

set -ex

if [ "$CLIPPY" = "yes" ]; then
    rustup component add clippy-preview
fi

if [ "$DOCS" = "yes" ]; then
    cargo install mdbook --force
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew update
    brew install gnuplot
fi

if [ "$RUSTFMT" = "yes" ]; then
    rustup component add rustfmt-preview
fi

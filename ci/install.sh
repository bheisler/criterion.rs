set -ex

if [ "$CLIPPY" = "yes" ]; then
    rustup component add clippy-preview
fi

if [ "$DOCS" = "yes" ]; then
    cargo install mdbook --no-default-features --force
    cargo install mdbook-linkcheck --force
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew unlink python@2 # because we're installing python3 and they both want to install stuff under /usr/local/Frameworks/Python.framework/
    brew install gnuplot
fi

if [ "$RUSTFMT" = "yes" ]; then
    rustup component add rustfmt-preview
fi

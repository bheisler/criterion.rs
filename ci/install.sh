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
    curl https://raw.githubusercontent.com/xd009642/tarpaulin/master/travis-install.sh | bash
fi

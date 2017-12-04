set -ex

if [ "$CLIPPY" = "yes" ]; then
  cargo install clippy --force
fi

if [ "$DOCS" = "yes" ]; then
    cargo install mdbook --force
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew update
    brew install gnuplot
fi

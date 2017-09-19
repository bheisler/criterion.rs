set -ex

cargo install clippy

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
  brew install gnuplot
fi

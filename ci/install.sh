set -ex

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
  cargo install clippy
fi

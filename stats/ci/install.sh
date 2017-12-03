# `install` phase: install stuff needed for the `script` phase

set -ex

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
    cargo install clippy --force
fi

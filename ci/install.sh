set -ex

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
  oldpath=$(pwd)
  cd $HOME
  git clone https://github.com/arcnmx/cargo-clippy
  cd $HOME/cargo-clippy
  cargo build --release
  cd $oldpath
fi

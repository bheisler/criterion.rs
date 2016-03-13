set -ex

oldpath=$(pwd)
cd $HOME
git clone https://github.com/arcnmx/cargo-clippy
cd $HOME/cargo-clippy
cargo build --release
cd $oldpath

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
  brew install gnuplot
fi

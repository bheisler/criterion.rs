# `install` phase: install stuff needed for the `script` phase

set -ex

oldpath=$(pwd)
cd $HOME
git clone https://github.com/arcnmx/cargo-clippy
cd $HOME/cargo-clippy
cargo build --release
cd $oldpath

case $TARGET in
  # Install standard libraries needed for cross compilation
  arm-unknown-linux-gnueabihf | \
  i686-apple-darwin | \
  i686-unknown-linux-gnu | \
  x86_64-unknown-linux-musl)
    if [ "$TARGET" = "arm-unknown-linux-gnueabihf" ]; then
      # information about the cross compiler
      arm-linux-gnueabihf-gcc -v

      # tell cargo which linker to use for cross compilation
      mkdir -p .cargo
      cat >.cargo/config <<EOF
[target.$TARGET]
linker = "arm-linux-gnueabihf-gcc"
EOF
    fi

    version=nightly
    tarball=rust-std-${version}-${TARGET}

    curl -Os http://static.rust-lang.org/dist/${tarball}.tar.gz

    tar xzf ${tarball}.tar.gz

    ${tarball}/install.sh --prefix=$(rustc --print sysroot)

    rm -r ${tarball}
    rm ${tarball}.tar.gz
    ;;
  # Nothing to do for native builds
  *)
    ;;
esac

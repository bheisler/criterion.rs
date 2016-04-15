# `install` phase: install stuff needed for the `script` phase

set -ex

case "$TRAVIS_OS_NAME" in
    linux)
        host=x86_64-unknown-linux-gnu
        ;;
    osx)
        host=x86_64-apple-darwin
        ;;
esac

mktempd() {
    echo $(mktemp -d 2>/dev/null || mktemp -d -t tmp)
}

install_rustup() {
    local td=$(mktempd)

    pushd $td
    curl -O https://static.rust-lang.org/rustup/dist/$host/rustup-setup
    chmod +x rustup-setup
    ./rustup-setup -y
    popd

    rm -r $td

    rustup default $CHANNEL
    rustc -V
    cargo -V
}

install_cargo_clippy() {
    pushd ~
    git clone --depth 1 https://github.com/arcnmx/cargo-clippy
    cd cargo-clippy
    cargo build --release
    ln -s target/release/cargo-clippy ~/.cargo/bin
    popd
}

install_std() {
    if [ "$host" != "$TARGET" ]; then
        rustup target add $TARGET
    fi
}

main() {
    install_rustup
    install_cargo_clippy
    install_std
}

main

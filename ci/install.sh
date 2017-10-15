set -ex

install_clippy() {
    # Attempt to install clippy
    ret=0
    cargo install clippy || ret=$?

    # If it failed (most likely already installed) then check versions and update if necessary.
    if [ 101 = $ret ]; then
        new_version=`cargo install clippy 2>&1 | grep "Installing clippy" | grep -o "[0-9.]*" || true`
        installed_version=`cargo install --list | grep "clippy v" | grep -o "[0-9.]*" || true`
        if [[ "$new_version" != "$installed_version" ]]; then
            cargo install clippy --force
        fi
    fi
}

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
    install_clippy
fi
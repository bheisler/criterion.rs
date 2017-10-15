set -ex

do_install() {
    app=$1

    # Attempt to install app
    ret=0
    cargo install $app || ret=$?

    # If it failed (most likely already installed) then check versions and update if necessary.
    if [ 101 = $ret ]; then
        new_version=`cargo install $app 2>&1 | grep "Installing $app" | grep -o "[0-9.]*" || true`
        installed_version=`cargo install --list | grep "$app v" | grep -o "[0-9.]*" || true`
        if [[ "$new_version" != "$installed_version" ]]; then
            cargo install $app --force
        fi
    fi
}

cargo install clippy --force
if [ "$TRAVIS_OS_NAME" = "linux"]; then
    do_install mdbook
fi

if [ "$TRAVIS_OS_NAME" = "osx" ] && [ "$GNUPLOT" = "yes" ]; then
    brew update
    brew install gnuplot
fi

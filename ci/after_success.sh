#!/bin/bash

set -ex

if [ "$TARGET" = "x86_64-unknown-linux-gnu" ]; then
    git clone --depth 1 https://github.com/davisp/ghp-import
    ./ghp-import/ghp_import.py -n target/$TARGET/doc
    set +x
    git push -fq "https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git" gh-pages && echo OK
    set -x
fi

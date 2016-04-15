#!/bin/bash

set -ex

main() {
    cargo build --target $TARGET
    cargo clippy --target $TARGET -- --cfg clippy
    cargo test --target $TARGET
    cargo doc --target $TARGET
}

main

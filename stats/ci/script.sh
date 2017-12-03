#!/bin/bash

set -ex

main() {
    cargo build
    cargo clippy -- --cfg clippy
    cargo test
    cargo doc
}

main

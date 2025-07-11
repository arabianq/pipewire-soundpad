#!/usr/bin/bash

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET=${1:-"x86_64-unknown-linux-gnu"}

bash ./build_scripts/build.sh "$RUST_TARGET"
cargo install cargo-deb
cargo deb --target "$RUST_TARGET"
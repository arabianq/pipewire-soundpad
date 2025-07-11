#!/usr/bin/bash

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET=${1:-"x86_64-unknown-linux-gnu"}

rustup target add "$RUST_TARGET"
cargo build --release --target "$RUST_TARGET"
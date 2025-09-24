#!/usr/bin/env bash

cd "$(dirname "$(realpath "$0")")/.." || exit

rm -rf ./target/debian

cargo install cargo-deb
cargo-deb
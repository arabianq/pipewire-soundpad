#!/usr/bin/env bash

cd "$(dirname "$(realpath "$0")")/.." || exit

rm -rf ./target/cargo-generate-rpm

cargo install cargo-generate-rpm
cargo-generate-rpm
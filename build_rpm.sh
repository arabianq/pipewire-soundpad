#!/usr/bin/sh

sh ./build.sh
cargo install cargo-generate-rpm
cargo generate-rpm
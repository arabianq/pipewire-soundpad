#!/usr/bin/bash

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET="x86_64-unknown-linux-gnu"

# Build all packages
bash ./build_scripts/build_deb.sh "$RUST_TARGET"
bash ./build_scripts/build_rpm.sh "$RUST_TARGET"

# Move .deb and .rpm packages to for_github_release directory
mkdir ./target/for_github_release
cp "$(find ./target/$RUST_TARGET/debian/pwsp_*_amd64.deb | sort -V | tail -n 1)" ./target/for_github_release/
cp "$(find ./target/generate-rpm/pwsp-*.x86_64.rpm | sort -V | tail -n 1)" ./target/for_github_release/

# Compress binaries using upx (if upx is installed)
if command -v upx >/dev/null 2>&1; then
  upx --best ./target/$RUST_TARGET/release/pwsp
  upx -t ./target/$RUST_TARGET/release/pwsp
fi

# Move binaries to for_github_release directory
cp ./target/$RUST_TARGET/release/pwsp ./target/for_github_release/pwsp-x86_64-linux-gnu

#!/usr/bin/env bash

cd "$(dirname "$(realpath "$0")")/.." || exit

bash ./scripts/build.sh

bash ./scripts/build_rpm.sh
bash ./scripts/build_deb.sh

if command -v upx >/dev/null 2>&1; then
  upx --best ./target/release/pwsp-gui
  upx --best ./target/release/pwsp-cli
  upx --best ./target/release/pwsp-daemon
  upx -t ./target/release/pwsp-gui
  upx -t ./target/release/pwsp-cli
  upx -t ./target/release/pwsp-daemon
fi

rm -rf ./target/for_github_release
mkdir ./target/for_github_release

cp "$(find ./target/debian/pwsp_*_amd64.deb | sort -V | tail -n 1)" ./target/for_github_release/
cp "$(find ./target/generate-rpm/pwsp-*.x86_64.rpm | sort -V | tail -n 1)" ./target/for_github_release/
zip -9j ./target/for_github_release/pwsp-x86_64-linux.zip ./target/release/pwsp-gui ./target/release/pwsp-cli ./target/release/pwsp-daemon
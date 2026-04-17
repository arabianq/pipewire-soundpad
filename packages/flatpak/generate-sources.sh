#!/bin/bash
set -e

if [ ! -f "Cargo.lock" ]; then
    echo "Error: Cargo.lock not found. Please run this script from the project root."
    return 1
fi

echo "Downloading flatpak-cargo-generator.py..."
curl -sLO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
chmod +x flatpak-cargo-generator.py

echo "Generating cargo-sources.json..."
python3 flatpak-cargo-generator.py Cargo.lock -o packages/flatpak/cargo-sources.json

echo "Cleaning up..."
rm flatpak-cargo-generator.py

echo "Successfully generated packages/flatpak/cargo-sources.json"

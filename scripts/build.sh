#!/usr/bin/env bash

cd "$(dirname "$(realpath "$0")")/.." || exit

cargo build --release
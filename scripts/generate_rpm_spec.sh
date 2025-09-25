#!/usr/bin/env bash

cd "$(dirname "$(realpath "$0")")/.." || exit

rust2rpm pwsp -a -o .
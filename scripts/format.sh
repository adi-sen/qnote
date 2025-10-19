#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

if [ "$1" = "--check" ]; then
    echo "Checking code formatting..."
    cargo +nightly fmt -- --check
else
    echo "Formatting code..."
    cargo +nightly fmt
fi

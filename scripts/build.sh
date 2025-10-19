#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

echo "Building debug..."
cargo build

echo "Building release..."
cargo build --release

echo ""
echo "Binary sizes:"
ls -lh target/debug/qnote target/release/qnote 2>/dev/null | awk '{print "  " $9 ": " $5}'

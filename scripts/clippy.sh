#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

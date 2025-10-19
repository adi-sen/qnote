#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

echo "Running pre-commit checks..."
echo ""

echo "[1/4] Checking format..."
cargo +nightly fmt -- --check

echo "[2/4] Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "[3/4] Running tests..."
cargo test --all-features

echo "[4/4] Checking build..."
cargo check --all-targets --all-features

echo ""
echo "All checks passed."

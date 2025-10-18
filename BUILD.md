# Building qnote

## Quick Build

```bash
# Small version (1.0MB) - requires system SQLite
cargo build --release

# Portable version (2.6MB) - works everywhere
cargo build --release --features bundled
```

## Version Comparison

| Version | Size | SQLite | Use Case |
|---------|------|--------|----------|
| **Small** | 1.0MB | System library (must be installed) | Personal use, macOS/Linux |
| **Portable** | 2.6MB | Bundled (self-contained) | Distribution, Windows, any system |

## SQLite Availability

- **macOS/Linux**: SQLite usually pre-installed ✓
- **Windows**: Requires manual installation for small version

## Distribution Recommendations

- **For end-users**: Use `--features bundled` (portable version)
- **For developers**: Either version works
- **For GitHub releases**: Provide both versions

## Examples

```bash
# Development (small is fine)
cargo build --release
./target/release/qnote

# Release build for distribution
cargo build --release --features bundled
strip target/release/qnote  # Optional: reduce by ~100KB
```

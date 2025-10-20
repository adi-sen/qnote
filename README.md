# qnote

> **Q**uick **Note** - A fast, lightweight terminal-based note-taking application written in Rust

[![Crates.io](https://img.shields.io/crates/v/qnote.svg)](https://crates.io/crates/qnote)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

Fast, lightweight note-taking with CLI and TUI interfaces. Tag-based organization with fuzzy search, external editor integration, and markdown preview.

## Quick Start

```bash
# Install
cargo install qnote

# Launch interactive TUI
qnote

# CLI usage
qnote add "My Note" "Content here" --tags work,ideas
qnote list
qnote search "keyword"
```

## Installation

<details>
<summary><b>From Crates.io</b> (Recommended)</summary>

```bash
cargo install qnote
```
</details>

<details>
<summary><b>Pre-built Binary</b></summary>

**Linux/macOS (one-liner):**
```bash
curl -sSL https://raw.githubusercontent.com/adi-sen/qnote/master/scripts/install.sh | bash
```

**Manual download:**
- Download from [Releases](https://github.com/adi-sen/qnote/releases)
- Extract and move to PATH

**Windows:**
- Download `qnote-Windows-x86_64.exe` from [Releases](https://github.com/adi-sen/qnote/releases)
- Rename to `qnote.exe` and add to PATH
</details>

<details>
<summary><b>From Source</b></summary>

```bash
git clone https://github.com/adi-sen/qnote.git
cd qnote
cargo install --path . --features bundled  # Portable version
```

**Build variants:**
- `--features bundled` - Bundled SQLite (portable, larger binary)
- Default - System SQLite (smaller, requires SQLite installed)
</details>

## Usage

### CLI Commands

> **[See full CLI examples and scripting guide](CLI_EXAMPLES.md)**

<details>
<summary><b>Basic Operations</b></summary>

```bash
# Create
qnote add <title> <content> [--tags tag1,tag2]

# Read
qnote list [--tag work] [--sort title] [--limit 10]
qnote show <id|pattern>

# Update
qnote edit <id|pattern> [--title "..."] [--content "..."]

# Delete
qnote delete <id|pattern> [--yes]
```
</details>

<details>
<summary><b>Search & Organization</b></summary>

```bash
# Search
qnote search "keyword"

# Tag management
qnote tags                # List all tags with counts
qnote list --tag work     # Filter by tag

# Sorting
qnote list --sort updated  # updated (default), created, title
```
</details>

<details>
<summary><b>Import/Export</b></summary>

```bash
# Export note to markdown
qnote export <id|pattern> [-o output.md]

# Import from markdown files
qnote import notes/*.md

# Statistics
qnote stats
```
</details>

### Interactive TUI

<details>
<summary><b>Keybindings</b></summary>

**Navigation:**
- `j/k` or `↓/↑` - Move selection
- `g/G` - Jump to top/bottom
- `Ctrl+j/k` - Scroll preview

**Actions:**
- `n` or `a` - New note
- `e` or `Enter` - Edit note
- `d` - Delete note
- `x` - Export to markdown
- `/` - Search mode
- `s` - Cycle sort mode
- `Esc` - Clear search/cancel
- `q` - Quit

**Search Mode:**
- Type to filter (fuzzy matching)
- `Ctrl+n/p` - Next/previous match
- `Enter` - Select note
- `Esc` - Exit search
</details>

## Configuration

<details>
<summary><b>Setup & Location</b></summary>

```bash
# Generate config file
qnote config

# View current config
qnote config --show
```

**Config locations:**
- Linux/BSD: `~/.config/qnote/config.toml`
- macOS: `~/Library/Application Support/qnote/config.toml`
- Windows: `%APPDATA%\qnote\config.toml`

**Database locations:**
- Linux: `~/.local/share/qnote/notes.db`
- macOS: `~/Library/Application Support/qnote/notes.db`
- Windows: `%APPDATA%\qnote\notes.db`
</details>

<details>
<summary><b>Configuration Options</b></summary>

```toml
[ui]
split_ratio = 0.4                    # List pane width (0.1-0.9)
message_display_keypresses = 5       # Status message duration
preview_scroll_step = 3              # Lines per scroll
preview_max_scroll_buffer = 10       # Preview scroll bounds
header_lines = 3                     # Preview header lines
max_markdown_formatting_buffer = 10  # Markdown formatting buffer

[editor]
default_editor = "nvim"              # Override $EDITOR (optional)
secure_temp_files = true             # 0600 permissions (Unix only)

[keybindings]
quit = "q"
new_note = "n"
delete = "d"
edit = "e"
search = "/"
export = "x"
sort = "s"
goto_top = "g"
goto_bottom = "G"
move_down = "j"
move_up = "k"

[database]
wal_mode = true                      # Write-Ahead Logging
cache_size_kb = -64000               # 64MB cache (negative = KB)
synchronous = "NORMAL"               # OFF, NORMAL, FULL, EXTRA
temp_store = "MEMORY"                # DEFAULT, FILE, MEMORY
```
</details>

<details>
<summary><b>Note Format</b></summary>

When editing notes in external editor:

```markdown
Note Title
#tag1 #tag2 #tag3

Note content goes here.
Multiple lines supported.
```

- **Line 1**: Title
- **Line 2**: Tags (optional, `#` prefix)
- **Line 3**: Blank separator
- **Line 4+**: Content
</details>

## Development

<details>
<summary><b>Project Structure</b></summary>

```
src/
├── main.rs
├── cli.rs              # CLI definitions
├── db.rs               # Database layer
├── commands/           # Command handlers
│   ├── note_ops.rs     # CRUD operations
│   ├── list.rs         # List, tags, stats
│   ├── io.rs           # Import/export
│   └── config.rs       # Config management
├── config/             # Configuration
│   ├── ui.rs
│   ├── keybindings.rs
│   ├── editor.rs
│   └── database.rs
├── utils/              # Utilities
│   ├── formatting.rs
│   ├── parsing.rs
│   ├── conversion.rs
│   └── interaction.rs
└── tui/                # Terminal UI
    ├── app.rs
    ├── render.rs
    ├── editor.rs
    └── markdown.rs
```
</details>

<details>
<summary><b>Building</b></summary>

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Portable version (bundled SQLite)
cargo build --release --features bundled

# Run tests
cargo test

# Format & lint
cargo fmt
cargo clippy
```
</details>

## Roadmap

### Core Functionality
- [x] Configuration file
- [ ] Backup/restore functionality

### TUI Improvements
- [ ] Tag filtering (filter notes by tag in TUI)
- [ ] Delete confirmation dialog
- [ ] Display note ID and creation date in preview
- [ ] Inline title editing (rename without external editor)
- [ ] Statistics/dashboard view
- [ ] Tag management view (list all tags with counts)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [clap](https://github.com/clap-rs/clap) - Command-line parser

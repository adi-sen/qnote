# qnote

A fast, lightweight terminal-based note-taking application written in Rust.

**qnote** = **Q**uick **Note** - your terminal-based note-taking companion.

## Features

- **Dual Interface**: Command-line interface for scripting and interactive TUI for browsing
- **Fast Search**: Fuzzy search across titles, content, and tags
- **Flexible Organization**: Tag your notes and sort by title, created date, or updated date
- **External Editor Integration**: Edit notes in your preferred editor ($EDITOR, or vi)
- **Markdown Support**: Basic markdown rendering in the preview pane
- **Lightweight**: ~1MB binary with system SQLite, ~2.6MB fully portable
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

### Binary Download (Recommended)

Download pre-built binaries from the [Releases](https://github.com/adi-sen/qnote/releases) page.

**Linux/macOS:**
```bash
# Download the appropriate binary for your platform
chmod +x qnote-*
sudo mv qnote-* /usr/local/bin/qnote
```

**Windows:**
- Download `qnote-windows-x86_64.exe`
- Rename to `qnote.exe`
- Add to PATH or run directly

### From Crates.io

```bash
cargo install qnote
```

### From Source

```bash
# Clone the repository
git clone https://github.com/adi-sen/qnote.git
cd qnote

# Build and install (portable version recommended)
cargo install --path . --features bundled
```

## Quick Start

```bash
# Launch interactive TUI (default)
qnote

# Or explicitly
qnote tui

# Add a note from command line
qnote add "My First Note" "This is the content of my note" --tags work,important

# List all notes
qnote list

# Search notes
qnote search "keyword"
```

## Usage

### Command-Line Interface

```bash
# Create a new note
qnote add <title> <content> [--tags tag1,tag2]

# List all notes
qnote list

# List notes with specific tag
qnote list --tag work

# Show a specific note
qnote show <id>

# Edit a note
qnote edit <id> --title "New Title"
qnote edit <id> --content "New content"
qnote edit <id> --tags tag1,tag2

# Delete a note
qnote delete <id>

# Search notes
qnote search <query>

# Launch TUI
qnote tui
```

### Interactive TUI

#### Navigation
- `j` / `↓` - Move down
- `k` / `↑` - Move up
- `g` - Jump to top
- `G` - Jump to bottom
- `Ctrl+j` - Scroll preview down
- `Ctrl+k` - Scroll preview up

#### Actions
- `n` / `a` - Create new note
- `e` / `Enter` - Edit selected note
- `d` - Delete selected note
- `x` - Export note to markdown file
- `/` - Enter search mode
- `Esc` - Clear search / cancel
- `s` - Cycle sort mode (updated, title, created)
- `q` - Quit

#### Search Mode
- Type to filter notes (fuzzy matching)
- `Ctrl+n` - Next match
- `Ctrl+p` - Previous match
- `Enter` - Select note
- `Esc` - Exit search

## Note Format

When creating or editing notes in your external editor, use this format:

```
Note Title
#tag1 #tag2 #tag3

Note content goes here.
You can use multiple lines.
```

- **Line 1**: Note title
- **Line 2**: (Optional) Tags prefixed with `#`
- **Line 3**: Blank line
- **Line 4+**: Note content

## Configuration

### Database Location

Notes are stored in:
- **Linux**: `~/.local/share/qnote/notes.db`
- **macOS**: `~/Library/Application Support/qnote/notes.db`
- **Windows**: `C:\Users\<User>\AppData\Roaming\qnote\notes.db`

### Editor Selection

qnote uses your preferred editor:
1. `$EDITOR` environment variable
2. `vi` (fallback)

Set your editor:
```bash
export EDITOR=nano  # or nvim, vim, code, etc.
```

## Building from Source

```bash
# Portable version (bundled SQLite, recommended for distribution)
cargo build --release --features bundled

# Small version (requires system SQLite)
cargo build --release
```

The bundled version (~2.6MB) is self-contained and works everywhere. The small version (~1MB) requires SQLite to be installed on the system.

## Development

### Dependencies
- Rust 2024 edition
- SQLite (if not using bundled feature)

### Project Structure
```
qnote/
├── src/
│   ├── main.rs       # Entry point
│   ├── cli.rs        # CLI commands
│   ├── db.rs         # Database operations
│   └── tui/          # Terminal UI
│       ├── app.rs    # Application state
│       ├── render.rs # UI rendering
│       ├── editor.rs # External editor integration
│       └── markdown.rs # Markdown rendering
├── Cargo.toml
└── README.md
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] Full markdown support
- [ ] Tag management UI
- [ ] Note templates
- [ ] Full-text search (SQLite FTS5)
- [ ] Note linking
- [ ] Configuration file
- [ ] Backup/restore functionality

## Acknowledgments

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [clap](https://github.com/clap-rs/clap) - Command-line parser

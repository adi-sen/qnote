# qnote CLI Examples (v1.0.1)

## Quick Reference

All major commands now support **title patterns** instead of IDs!

```bash
qnote show "shopping"      # Find and show by title
qnote edit "meeting" -c "Updated notes"
qnote delete "old" --yes   # Skip confirmation
```

## Basic Usage

### Creating Notes
```bash
# Create a note
qnote add "Shopping List" "Milk, eggs, bread" --tags personal,todo

# Without tags
qnote add "Meeting Notes" "Discussed project timeline"
```

### Listing Notes
```bash
# List all notes (detailed)
qnote list

# List notes with specific tag
qnote list --tag work

# Compact format (one line per note)
qnote list --oneline
# Output: 42	Shopping List [personal, todo]
#         38	Meeting Notes

# Sort by different fields
qnote list --sort title     # Alphabetical by title
qnote list --sort created   # Newest created first
qnote list --sort updated   # Most recently updated (default)

# Limit results
qnote list --limit 5        # Show only 5 most recent
qnote list --tag work --limit 10 --sort title
```

### Viewing Notes

The new hybrid approach accepts **either ID or title pattern**:

```bash
# By ID (traditional)
qnote show 42

# By title pattern (NEW!)
qnote show "shopping"       # Finds "Shopping List"
qnote show "meeting"        # Finds "Meeting Notes"

# If multiple matches, shows options:
qnote show "work"
# Multiple notes found matching 'work':
#   [12] Work TODO
#   [15] Work meeting notes
#   [18] Workspace setup
# Please specify a more specific pattern or use the exact ID
```

### Editing Notes

Same hybrid approach for edit:

```bash
# Edit by title pattern
qnote edit "shopping" --content "Milk, eggs, bread, cheese"

# Edit by ID
qnote edit 42 --title "Grocery Shopping"

# Edit multiple fields
qnote edit "meeting" --title "Team Meeting" --tags work,important
```

### Deleting Notes

Delete with confirmation prompt:

```bash
# By title pattern (asks for confirmation)
qnote delete "shopping"
# Found: [42] Shopping List
# Delete this note? (y/N):

# By ID with confirmation
qnote delete 42

# Skip confirmation with --yes flag
qnote delete "shopping" --yes
qnote delete 42 -y
```

### Searching Notes
```bash
# Search across titles, content, and tags
qnote search "project"
qnote search "todo"
```

### Export & Import

Export notes to markdown files:

```bash
# Export by title pattern
qnote export "shopping"
# Creates: Shopping_List.md

# Export with custom filename
qnote export "meeting" -o team_meeting.md
qnote export 42 --output important.md

# Export all notes (loop)
qnote list --oneline | cut -f1 | while read id; do
    qnote export "$id"
done
```

Import notes from markdown files:

```bash
# Import single file
qnote import my_note.md

# Import multiple files
qnote import notes/*.md
qnote import note1.md note2.md note3.md

# Markdown file format:
# Line 1: Note Title
# Line 2 (optional): #tag1 #tag2
# Line 3+: Note content
```

### Tag Management

```bash
# List all tags with counts
qnote tags
# Output:
# Tags (5 total):
#   work (12)
#   personal (8)
#   todo (5)
#   important (3)
#   project (2)

# Combine with grep for searching
qnote tags | grep work
```

### Statistics

```bash
# Show note statistics
qnote stats
# Output:
# ==================================================
# qnote Statistics
# ==================================================
# Total notes:      42
# Unique tags:      12
# Total size:       15.3 KB
# Oldest note:      Project Ideas (2024-01-15)
# Most recent:      Shopping List (2025-09-18 14:30)
# ==================================================
```

### Configuration

```bash
# Generate default configuration file
qnote config
# Generates: ~/.config/qnote/config.toml (Linux/BSD)
#            ~/Library/Application Support/qnote/config.toml (macOS)
#            %APPDATA%\qnote\config.toml (Windows)

# View current configuration
qnote config --show

# After generating, edit with your preferred editor
$EDITOR ~/.config/qnote/config.toml

# Config options include:
# - UI settings (split ratio, scroll speed, colors)
# - Keybindings (customize all TUI shortcuts)
# - Editor preferences (default editor, temp file security)
# - Database settings (WAL mode, cache size, performance tuning)
```

## fzf Integration

The `--oneline` format is designed for piping to fzf and other tools:

### Interactive Note Selection
```bash
# View a note with fzf picker
qnote list --oneline | fzf | cut -f1 | xargs qnote show

# Edit a note with fzf picker
qnote list --oneline | fzf | cut -f1 | xargs -I {} qnote edit {} --content "Updated"

# Delete a note with fzf picker
qnote list --oneline | fzf | cut -f1 | xargs qnote delete
```

### Shell Aliases

Add these to your `.bashrc` or `.zshrc`:

```bash
# Interactive note viewer
alias qnv='qnote list --oneline | fzf --preview "qnote show {1}" | cut -f1 | xargs qnote show'

# Interactive note editor (opens in $EDITOR)
alias qne='qnote list --oneline | fzf --preview "qnote show {1}" | cut -f1 | xargs -I {} sh -c "qnote show {} > /tmp/note.md && $EDITOR /tmp/note.md"'

# Interactive note deletion
alias qnd='qnote list --oneline | fzf --preview "qnote show {1}" | cut -f1 | xargs qnote delete'

# Quick search with fzf
alias qns='qnote search'
```

### Advanced fzf Examples

```bash
# Filter by tag, then select with fzf
qnote list --tag work --oneline | fzf

# Search and preview
qnote list --oneline | fzf --preview 'qnote show {1}' --preview-window=right:60%

# Multi-select and delete
qnote list --oneline | fzf -m | cut -f1 | xargs -I {} qnote delete {} --yes
```

## Scripting Examples

### Backup All Notes
```bash
#!/bin/bash
mkdir -p notes_backup
qnote list --oneline | while IFS=$'\t' read -r id title; do
    filename="notes_backup/${id}-$(echo "$title" | tr ' ' '_').md"
    qnote show "$id" > "$filename"
done
```

### Export Notes with Specific Tag
```bash
qnote list --tag work --oneline | cut -f1 | while read -r id; do
    qnote show "$id" > "work_note_${id}.md"
done
```

### Count Notes by Tag
```bash
echo "work: $(qnote list --tag work --oneline | wc -l)"
echo "personal: $(qnote list --tag personal --oneline | wc -l)"
```

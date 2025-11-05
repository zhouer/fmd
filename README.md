# fmd — Find Markdown files by metadata

**A command-line tool that understands your notes.**

Search Markdown files by tags, frontmatter, and custom fields. Built for note-takers who organize with metadata.

```bash
fmd --tag python              # Find all notes tagged with python
fmd --title "meeting"         # Find notes with "meeting" in title
fmd --tag work --tag urgent   # Find notes with work OR urgent tags
fmd --field "author:John"     # Find notes by John
```

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)

---

## Features

- **🗂 Metadata-aware** — Understands YAML frontmatter and inline `#tags`
- **🔍 Flexible filtering** — By tag, title, filename, or any custom field
- **🧩 Unix-friendly** — Compose with `xargs`, `grep`, `fzf`
- **🎯 Zero-config** — No database, no indexing required
- **⚡ Fast** — Parallel processing with Rust

---

## Quick Start

### Installation

```bash
# From source (requires Rust)
cargo install --path .

# Or use the install script
./install-rust.sh
```

### Basic Usage

```bash
# List all Markdown files
fmd

# Find by tag
fmd --tag python
fmd -t linux -t macos        # OR logic: linux OR macos

# Find by title (YAML frontmatter or # heading)
fmd --title "meeting notes"
fmd -T "project.*2025"       # Regex supported

# Find by filename
fmd --name "2025-01"         # Files with "2025-01" in name
fmd -i -n readme             # Case-insensitive

# Find by custom field
fmd --field "author:John"
fmd -f "status:draft"

# Combine filters (AND logic across types)
fmd -t work -T "meeting" -f "author:John"
# → (tag=work) AND (title=meeting) AND (author=John)

# Search entire file content (not just first 10 lines)
fmd -t project --full-text
```

---

## Metadata Format Support

fmd understands **two metadata formats**:

### 1. YAML Frontmatter

```markdown
---
title: My Note
tags: [python, rust, cli]
author: John Doe
date: 2025-01-15
status: draft
---

# Content starts here
```

**Multi-line format:**
```markdown
---
title: Setup Guide
tags:
  - linux
  - server
  - tutorial
date: 2025-01-15
---
```

### 2. Inline Format

```markdown
# My Document

tags: #python #rust #cli
author: John Doe
```

**Note:** By default, fmd scans the first 10 lines for inline metadata. Use `--full-text` to search the entire file.

---

## Usage Examples

### Search by Tags

```bash
# Single tag
fmd -t python

# Multiple tags (OR logic)
fmd -t python -t rust        # python OR rust

# Full-text tag search (searches #tag in entire file)
fmd -t project --full-text
```

### Search by Title

```bash
# Find notes with "meeting" in title
fmd -T meeting

# Regex patterns supported
fmd -T "notes.*2025"
```

### Search by Filename

```bash
# Case-sensitive by default
fmd -n "2025-01"

# Case-insensitive
fmd -i -n readme
```

### Search by Custom Fields

```bash
# By author
fmd -f "author:John"

# By status
fmd -f "status:draft"

# By date
fmd -f "date:2025-01"
```

### Combine Filters

Filters of **different types** use **AND** logic:

```bash
# Tag AND title
fmd -t work -T meeting

# Tag AND author AND status
fmd -t project -f "author:John" -f "status:active"
```

Filters of the **same type** use **OR** logic:

```bash
# tag=python OR tag=rust
fmd -t python -t rust

# author=John OR author=Jane
fmd -f "author:John" -f "author:Jane"
```

### Compose with Unix Tools

```bash
# Search file contents
fmd -t finance | xargs grep -l "Apple"

# Edit all drafts
fmd -f "status:draft" | xargs $EDITOR

# Count files
fmd -t todo | wc -l

# Interactive selection with fzf
fmd | fzf --preview 'bat {}' | xargs $EDITOR

# Move files (safe with spaces)
fmd -0 -t linux | xargs -0 -I {} mv {} ./topics/linux/

# Create archive
fmd -t archive | xargs tar -czf archive.tar.gz
```

---

## Command-Line Options

| Option | Description |
|--------|-------------|
| `-0` | NUL-delimited output (safe for filenames with spaces) |
| `-i, --ignore-case` | Case-insensitive filename matching |
| `-d, --depth N` | Limit search depth (1=current dir only) |
| `-t, --tag TAG` | Filter by tag (case-insensitive) |
| `-T, --title PAT` | Filter by title (case-insensitive, regex) |
| `-n, --name PAT` | Filter by filename (regex) |
| `-f, --field F:P` | Filter by frontmatter field (format: `field:pattern`) |
| `--glob GLOB` | File pattern to match (default: `**/*.md`) |
| `--head N` | Lines to scan for metadata (default: 10) |
| `--full-text` | Search entire file content |
| `-h, --help` | Show help message |

---

## Filter Logic

### Same Type → OR

```bash
fmd -t A -t B              # A OR B
fmd -f author:X -f author:Y # X OR Y
```

### Different Types → AND

```bash
fmd -t A -T B              # A AND B
fmd -t A -f status:draft   # A AND draft
```

### Complex Example

```bash
# (tag=work OR tag=personal) AND title=meeting AND author=John
fmd -t work -t personal -T meeting -f "author:John"
```

---

## Full-Text Search

By default, fmd scans only the **first 10 lines** for inline tags (controlled by `--head`). Use `--full-text` to search the entire file:

```bash
fmd -t project                 # YAML + inline tags in first 10 lines
fmd -t project --full-text     # Anywhere in content
```

| Mode | YAML `tags:` | Inline `tags:` | Content `#tag` |
|------|--------------|----------------|----------------|
| Default | ✓ | ✓ (first 10 lines) | ✗ |
| `--full-text` | ✓ | ✓ (entire file) | ✓ |

---

## Advanced Examples

```bash
# Find finance notes mentioning Apple
fmd -t finance | xargs grep -l "Apple"

# Move all linux notes from current directory
fmd -d 1 -t linux | xargs -I {} mv {} ./topics/linux/

# Edit all draft files
fmd -f "status:draft" | xargs $EDITOR

# Backup November files
fmd -f "date:2025-11" | xargs -I {} cp {} ./backup/

# Find beginner tutorials
fmd -f "category:tutorial" -f "difficulty:beginner"

# Linux guides updated in 2025
fmd -t linux -f "updated:2025" -T "guide"

# Interactive selection
fmd -t project | fzf --preview 'bat --color=always {}' | xargs $EDITOR

# Safe handling of filenames with spaces
fmd -0 -t important | xargs -0 ls -lh
```

---

## Building from Source

### Requirements

- Rust 1.91 or later (install from [rustup.rs](https://rustup.rs/))

### Install

```bash
# Recommended: Install to ~/.cargo/bin
cargo install --path .

# Or use the install script
./install-rust.sh

# Alternative: Install to ~/.local/bin
./install-rust.sh local
```

### Manual Build

```bash
cargo build --release
./target/release/fmd --help
```

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## License

MIT License © 2025 Enjan Chou

---

**Find Markdown files by what matters: metadata.**

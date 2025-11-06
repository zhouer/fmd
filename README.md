# fmd — Find Markdown files by metadata

**A command-line tool that understands your notes.**

Search Markdown files by tags, frontmatter, and custom fields. Built for note-takers who organize with metadata.

```bash
fmd --tag python              # Find all notes tagged with python
fmd --title "meeting"         # Find notes with "meeting" in title
fmd --tag work --tag urgent   # Find notes with work OR urgent tags
fmd --author "John"           # Find notes by John
```

[![CI](https://github.com/zhouer/fmd/workflows/CI/badge.svg)](https://github.com/zhouer/fmd/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/zhouer/fmd/branch/master/graph/badge.svg)](https://codecov.io/gh/zhouer/fmd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)

---

## Features

- **🗂 Metadata-aware** — Understands YAML frontmatter and inline `#tags`
- **🔍 Flexible filtering** — By tag, title, filename, date, or any custom field
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

# Find by author
fmd --author "John"
fmd -a "Jane" -a "Bob"       # OR logic: Jane OR Bob

# Find by filename
fmd --name "2025-01"         # Files with "2025-01" in name
fmd -i -n readme             # Case-insensitive

# Find by custom field
fmd --field "author:John"
fmd -f "status:draft"

# Find by date range
fmd --date-after 2025-01-01           # Files dated after Jan 1, 2025
fmd --date-before 2025-12-31          # Files dated before Dec 31, 2025
fmd --date-after 2025-01-01 --date-before 2025-03-31  # Q1 2025

# Combine filters (AND logic across types)
fmd -t work -T "meeting" -a "John"
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
date: 2025-01-15
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

### Search by Author

```bash
# Single author
fmd --author "John"
fmd -a "John Doe"

# Multiple authors (OR logic)
fmd -a "John" -a "Jane"      # John OR Jane

# Case-insensitive matching
fmd -a "john doe"            # Matches "John Doe"

# Partial matching
fmd -a "Doe"                 # Matches "John Doe", "Jane Doe", etc.
```

### Search by Custom Fields

```bash
# By status
fmd -f "status:draft"

# By date (partial match)
fmd -f "date:2025-01"
```

### Search by Date Range

Date filtering checks the `date`, `created`, `updated`, and `modified` fields. A file matches if **any** of these dates satisfies the filter.

```bash
# Files from 2025 onwards
fmd --date-after 2025-01-01

# Files before a specific date
fmd --date-before 2025-06-30

# Date range (Q1 2025)
fmd --date-after 2025-01-01 --date-before 2025-03-31

# Recent notes (last month)
fmd --date-after 2025-10-01

# Combine with other filters
fmd -t work --date-after 2025-01-01  # Work notes from 2025
```

**Supported date fields** (checked in order):
- `date:` — Primary date field
- `created:` — Creation date
- `updated:` — Last update date
- `modified:` — Last modification date

**Date format:** `YYYY-MM-DD` (ISO 8601)

### Combine Filters

Filters of **different types** use **AND** logic:

```bash
# Tag AND title
fmd -t work -T meeting

# Tag AND author AND status
fmd -t project -a "John" -f "status:active"
```

Filters of the **same type** use **OR** logic:

```bash
# tag=python OR tag=rust
fmd -t python -t rust

# author=John OR author=Jane
fmd -a "John" -a "Jane"
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
| `-a, --author PAT` | Filter by author (case-insensitive) |
| `-n, --name PAT` | Filter by filename (regex) |
| `-f, --field F:P` | Filter by frontmatter field (format: `field:pattern`) |
| `--date-after DATE` | Filter files with dates on or after DATE (format: YYYY-MM-DD) |
| `--date-before DATE` | Filter files with dates on or before DATE (format: YYYY-MM-DD) |
| `--glob GLOB` | File pattern to match (default: `**/*.md`) |
| `--head N` | Lines to scan for metadata (default: 10) |
| `--full-text` | Search entire file content |
| `-v, --verbose` | Show verbose output including warnings and errors |
| `-h, --help` | Show help message |

---

## Filter Logic

### Same Type → OR

```bash
fmd -t A -t B              # A OR B
fmd -a X -a Y              # X OR Y
```

### Different Types → AND

```bash
fmd -t A -T B              # A AND B
fmd -t A -f status:draft   # A AND draft
```

### Complex Example

```bash
# (tag=work OR tag=personal) AND title=meeting AND author=John
fmd -t work -t personal -T meeting -a "John"
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

# Backup recent files (last 3 months)
fmd --date-after 2025-08-01 | xargs -I {} cp {} ./backup/

# Archive old notes (before 2024)
fmd --date-before 2023-12-31 | xargs -I {} mv {} ./archive/

# Find Q1 2025 work notes
fmd -t work --date-after 2025-01-01 --date-before 2025-03-31

# Find beginner tutorials
fmd -f "category:tutorial" -f "difficulty:beginner"

# Recent meeting notes
fmd -T meeting --date-after 2025-10-01

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

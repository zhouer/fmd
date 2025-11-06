# fmd — Find Markdown files by metadata

**The command-line companion for your knowledge base.**

**fmd** finds Markdown files by tags, frontmatter, and custom metadata. Pipe the results to `xargs` to move notes by tags, to `grep` to search content, or to any Unix tool to build custom workflows. If you organize notes with metadata, fmd gives you the power to act on it.

[![CI](https://github.com/zhouer/fmd/workflows/CI/badge.svg)](https://github.com/zhouer/fmd/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/zhouer/fmd/branch/master/graph/badge.svg)](https://codecov.io/gh/zhouer/fmd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)

---

## Features

- **🗂 Metadata-aware** — Understands YAML frontmatter and inline `#tags`
- **🔍 Flexible filtering** — By tag, title, filename, date, or any custom field
- **🧩 Unix-friendly** — Compose with `xargs`, `grep`, `fzf`
- **🎯 Zero-setup** — No database, no indexing required
- **⚡ Fast** — Smart filtering, reads metadata only, parallel processing

---

## Quick Start

### Installation

```bash
# From crates.io (recommended)
cargo install fmd

# From source
cargo install --path .
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

author: John Doe
date: 2025-01-15
tags: #python #rust #cli
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

### Combining Filters

Filters of the same type use **OR** logic, while different types use **AND** logic:

#### Same Type → OR

```bash
fmd -t A -t B              # A OR B
fmd -a X -a Y              # X OR Y
```

#### Different Types → AND

```bash
fmd -t A -T B              # A AND B
fmd -t A -f status:draft   # A AND draft
```

#### Complex Example

```bash
# (tag=work OR tag=personal) AND title=meeting AND author=John
fmd -t work -t personal -T meeting -a "John"
```

### Full-Text Search

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

## Command-Line Options

| Option | Description |
|--------|-------------|
| `-t, --tag TAG` | Filter by tag (case-insensitive) |
| `-T, --title PAT` | Filter by title (case-insensitive, regex) |
| `-a, --author PAT` | Filter by author (case-insensitive) |
| `-n, --name PAT` | Filter by filename (regex) |
| `-f, --field F:P` | Filter by frontmatter field (format: `field:pattern`) |
| `--date-after DATE` | Filter files with dates on or after DATE (format: YYYY-MM-DD) |
| `--date-before DATE` | Filter files with dates on or before DATE (format: YYYY-MM-DD) |
| `--glob GLOB` | File pattern to match (default: `**/*.md`) |
| `-d, --depth N` | Limit search depth (1=current dir only) |
| `--head N` | Lines to scan for metadata (default: 10) |
| `--full-text` | Search entire file content |
| `-i, --ignore-case` | Case-insensitive matching for `--name` filter |
| `-0` | NUL-delimited output (safe for filenames with spaces) |
| `-v, --verbose` | Show verbose output including warnings and errors |
| `-h, --help` | Show help message |

---

## Usage with Unix Tools

fmd is designed to work seamlessly with standard Unix tools. Here are practical examples:

**Note:** When filenames contain spaces, use the `-0` option with `xargs -0` for safe processing:
```bash
fmd -0 -t project | xargs -0 command
```

### Search and Edit

```bash
# Search file contents
fmd -t finance | xargs grep -l "Apple"

# Edit all draft files
fmd -f "status:draft" | xargs $EDITOR

# Interactive selection with fzf
fmd -t project | fzf --preview 'bat --color=always {}' | xargs $EDITOR
```

### File Management

```bash
# Move files (safe with spaces using -0)
fmd -0 -t linux | xargs -0 -I {} mv {} ./topics/linux/

# Move files from current directory only
fmd -d 1 -t linux | xargs -I {} mv {} ./topics/linux/

# Backup recent files (last 3 months)
fmd --date-after 2025-08-01 | xargs -I {} cp {} ./backup/

# Archive old notes (before 2024)
fmd --date-before 2023-12-31 | xargs -I {} mv {} ./archive/

# Create tar archive
fmd -t archive | xargs tar -czf archive.tar.gz
```

### Analysis and Reporting

```bash
# Count files by tag
fmd -t todo | wc -l

# List file details (safe with spaces)
fmd -0 -t important | xargs -0 ls -lh

# Find Q1 2025 work notes
fmd -t work --date-after 2025-01-01 --date-before 2025-03-31

# Find beginner tutorials
fmd -f "category:tutorial" -f "difficulty:beginner"

# Recent meeting notes
fmd -T meeting --date-after 2025-10-01
```

---

## Building from Source

**Requirements:** Rust 1.91+ ([install from rustup.rs](https://rustup.rs/))

```bash
# Clone and install
git clone https://github.com/zhouer/fmd.git
cd fmd
cargo install --path .

# Or just build without installing
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

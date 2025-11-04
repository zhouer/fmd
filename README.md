# mdq — Markdown Query Tool

**`find` for Markdown files with metadata awareness.**

A Unix-style command-line tool for querying Markdown/Obsidian notes by tags, title, and filename. Outputs filenames to stdout, perfect for piping to `xargs`, `grep`, `fzf`, and other Unix tools.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Bash](https://img.shields.io/badge/bash-4.0%2B-green.svg)](https://www.gnu.org/software/bash/)

---

## Features

- **🗂 Metadata-aware** — Understands Obsidian-style metadata (`tags:`, `# heading`)
- **📝 Full YAML frontmatter support** — Query any frontmatter field (author, date, status, etc.)
- **🔍 Flexible filtering** — Filter by tag, title, filename, or custom fields with regex support
- **🧩 Composable** — Pipe output to `xargs`, `grep`, `fzf`, and other Unix tools
- **🛡 Safe & predictable** — Only lists files, never modifies them
- **🪶 Zero dependencies** — Pure Bash, portable across Linux & macOS

---

## Installation


```bash
# Download and install
curl -o mdq https://raw.githubusercontent.com/yourname/mdq/main/mdq
chmod +x mdq
sudo mv mdq /usr/local/bin/

# Verify installation
mdq --help
```

**Requirements:** Bash 4.0+, standard Unix utilities (`find`, `awk`, `grep`)

---

## Quick Start

```bash
# List all Markdown files recursively
mdq

# Find files with specific tags
mdq -t Linux -t macOS

# Find files by author
mdq -f "author:John"

# Combine filters: tag=Work AND status=draft
mdq -t Work -f "status:draft"

# Pipe to other tools
mdq -t Finance | xargs grep -l "Apple"
```

---

## Usage

### Metadata Format Support

`mdq` supports **two metadata formats**:

**Simple inline format:**
```markdown
# My Document Title
tags: #Linux #macOS #CLI
```

**YAML frontmatter format:**
```markdown
---
title: My Document Title
tags: [Linux, macOS, CLI]
author: John Doe
date: 2025-11-04
status: draft
category: Technology
---
```

Or YAML array format:
```markdown
---
title: Setup Guide
tags:
  - Linux
  - Server
  - Tutorial
author: Jane Smith
---
```

### List Markdown files

```bash
mdq                           # list all *.md recursively from current directory
mdq ./notes                   # list recursively from specific directory
mdq -d 1                      # list only current directory (non-recursive)
mdq -d 1 ./notes              # list only in ./notes directory (non-recursive)
```

### Filter by tags

```bash
mdq -t Linux                  # files tagged #Linux (recursive)
mdq -t Finance -t Apple       # files with tag #Finance OR #Apple (OR logic)
mdq -d 1 -t Linux             # search only current directory for #Linux tagged files
mdq -t Linux ./notes          # search recursively from ./notes for #Linux files
```

### Filter by title (markdown heading or YAML title)

```bash
mdq -T "Meeting"              # files with "Meeting" in title (YAML or # heading)
mdq -T "Notes.*2025"          # supports regex patterns
mdq -d 2 -T "Project"         # search up to 2 levels deep by title
```

### Filter by filename

```bash
mdq -n "2025"                 # matches files like "2025-01-15 Meeting.md" (case-sensitive)
mdq -n "^2025-11"             # files starting with "2025-11"
mdq -i -n "readme"            # case-insensitive: matches README.md, readme.md, ReadMe.md
mdq -d 1 -n "\.draft\.md$"    # all draft files in current directory only
```

### Filter by custom frontmatter fields

```bash
mdq -f "author:John"          # files where author contains "John"
mdq -f "status:draft"         # files with status = "draft"
mdq -f "date:2025-11"         # files with date containing "2025-11"
mdq -f "category:Tech"        # files in Technology category
```

### Combine filters (AND logic across types)

```bash
mdq -t Linux -T "Setup"                      # tag=#Linux AND title="Setup"
mdq -t Work -n "2025"                        # tag=Work AND filename contains "2025"
mdq -t Work -t Personal -T "2025"            # (tag=Work OR tag=Personal) AND title="2025"
mdq -f "author:John" -f "author:Jane"        # author=John OR author=Jane
mdq -t Linux -f "status:draft"               # tag=Linux AND status=draft
mdq -t Finance -T "Report" -f "date:2025"    # tag=Finance AND title=Report AND date=2025
```

### Pipe to Unix Tools

`mdq` follows the Unix philosophy of composability:

```bash
# Search file contents
mdq -t Finance | xargs grep -l 'Apple'

# Move files
mdq -d 1 -t Linux | xargs -I {} mv {} ./topics/Linux/

# Count files
mdq -t TODO | wc -l

# Interactive selection with fzf
mdq | fzf --preview 'bat {}' | xargs $EDITOR

# Create archive
mdq -t Archive | xargs tar -czf archive.tar.gz

# Safe handling of filenames with spaces
mdq -0 -t Linux | xargs -0 -I {} mv {} ./topics/Linux/
```

---

## Command-Line Options

| Option | Description |
|--------|-------------|
| `-0` | Use NUL-delimited output (safe for filenames with spaces) |
| `-i, --ignore-case` | Case-insensitive filename matching (tags/title/fields already case-insensitive) |
| `-d, --depth N` | Limit search depth (1=current dir only, default: recursive) |
| `-t, --tag TAG` | Filter by tag (searches `tags:` metadata, case-insensitive) |
| `-T, --title PAT` | Filter by title (searches YAML `title:` or `# heading`, case-insensitive) |
| `-n, --name PAT` | Filter by filename (regex pattern, case-sensitive by default, use `-i` for case-insensitive) |
| `-f, --field F:P` | Filter by frontmatter field (format: `field:pattern`, case-insensitive) |
| `--glob GLOB` | File pattern to match (default: `*.md`) |
| `--head N` | Lines to scan for metadata (default: 10) |
| `-h, --help` | Show help message |

---

## Filter Logic

`mdq` uses intuitive filter logic:

### Same Type: OR Logic
Multiple filters of the same type are combined with **OR**:
- `-t A -t B` → tag A **OR** tag B
- `-f author:John -f author:Jane` → author John **OR** Jane

### Cross Type: AND Logic
Filters of different types are combined with **AND**:
- `-t A -T X` → tag A **AND** title X
- `-t A -f status:draft` → tag A **AND** status=draft

### Complex Combinations
```bash
# (tag=Work OR tag=Personal) AND title=Meeting AND author=John
mdq -t Work -t Personal -T "Meeting" -f "author:John"
```

---

## Design Philosophy

> **"`find` for Markdown — metadata is the new filesystem."**

Traditional `find` filters by filename, path, or mtime. `mdq` extends this paradigm to **Markdown metadata** — enabling queries by tag, title, author, and custom frontmatter fields without requiring a database.

### Design Principles

1. **Do one thing well** — Only list files, never modify them
2. **Composable** — Output filenames for other tools to process
3. **Predictable** — Behave like `find` (recursive by default)
4. **Metadata-aware** — Understand Obsidian-style tags and YAML frontmatter
5. **Flexible** — Support depth control, custom fields, and regex patterns
6. **Portable** — Pure Bash with no external dependencies

---

## Advanced Examples

```bash
# Find notes tagged #Finance mentioning Apple (recursive)
mdq -t Finance | xargs grep -l 'Apple'

# Move all Linux-tagged notes from current directory only
mdq -d 1 -t Linux | xargs -I {} mv {} ./topics/Linux/

# Count all TODO files recursively
mdq -t TODO | wc -l

# Filter by title pattern (YAML title or heading inside file)
mdq -T "Meeting Notes" | xargs -I {} mv {} ./meetings/

# Filter by filename pattern
mdq -n "2025-11" | xargs -I {} mv {} ./archive/november/

# Filter by custom frontmatter fields
mdq -f "author:John" | wc -l                    # count John's notes
mdq -f "status:draft" | xargs $EDITOR           # edit all draft files
mdq -f "date:2025-11" | xargs -I {} cp {} ./backup/  # backup November files

# Combine tag and title filters (AND logic across types)
mdq -t Important -T "Urgent"        # tag=Important AND title=Urgent

# Combine tag and filename filters
mdq -t Finance -n "2025"            # tag=Finance AND filename contains "2025"

# Combine tag and frontmatter filters
mdq -t Project -f "status:active"   # tag=Project AND status=active
mdq -f "author:John" -f "status:published"  # (author=John OR status=published)

# Multiple filters: OR within type, AND across types
mdq -t Finance -t Tech -T "2025.*Report"  # (tag=Finance OR tag=Tech) AND title="2025.*Report"
mdq -t Work -f "author:John" -f "author:Jane" -T "Meeting"  # tag=Work AND (author=John OR Jane) AND title=Meeting

# Interactive file selection with fzf
mdq | fzf --preview 'bat {}' | xargs $EDITOR

# Work safely with null-delimited filenames (handles spaces)
mdq -0 -t Linux | xargs -0 -I {} mv {} ./topics/Linux/

# Combine with other find-like patterns
mdq -n "draft" | xargs rm -i                      # delete draft files (with confirmation)
mdq -t Archive | xargs tar -czf archive.tar.gz    # create tarball

# Case-insensitive filename matching
mdq -i -n "readme"                  # matches README.md, readme.md, ReadMe.md
mdq -i -n "todo" -t urgent          # case-insensitive filename with tag filter

# Limit search depth
mdq -d 2 -t Project                 # search only 2 levels deep
mdq -d 1 | wc -l                    # count files in current directory only

# Advanced YAML frontmatter queries
mdq -f "category:Tutorial" -f "difficulty:beginner"  # beginner tutorials OR easy difficulty
mdq -t Linux -f "updated:2025" -T "Guide"            # Linux guides updated in 2025
```

---

## Roadmap

- [ ] Date-based filtering with comparison operators (`--date-after`, `--date-before`)
- [ ] Shell completion (zsh, fish, bash)
- [ ] Performance: optional metadata cache
- [ ] Support for nested YAML structures
- [ ] Boolean operators in filter expressions

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

---

## License

MIT License © 2025 Enjan Chou

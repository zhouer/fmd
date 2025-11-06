# fmd Feature Roadmap

This document outlines planned features and improvements for fmd.

---

## Planned Features

### 1. Sorting Options ⭐⭐⭐⭐
**Status**: TBD
**Impact**: High - Improves usability
**Complexity**: Low

#### Proposed API
```bash
# Sort by different fields
fmd -t Work --sort date         # By frontmatter date
fmd -t Work --sort title        # Alphabetically by title
fmd -t Work --sort modified     # By file modification time
fmd -t Work --sort path         # By file path
fmd -t Work --sort created      # By file creation time

# Reverse order
fmd -t Work --sort date --reverse
```

#### Implementation Notes
- Add `--sort` enum argument: `Date`, `Title`, `Modified`, `Path`, `Created`
- Add `--reverse` boolean flag
- Sort results before output
- Use file system metadata for `modified` and `created`

---

### 2. Interactive Mode ⭐⭐⭐⭐
**Status**: TBD
**Impact**: Medium
**Complexity**: Medium-High

#### Proposed API
```bash
# Launch interactive TUI
fmd --interactive

# Integration with fzf
fmd -t Work --fzf

# Preview mode
fmd --fzf --preview
```

#### Implementation Notes
- Use `ratatui` or `crossterm` for TUI
- Provide real-time filtering
- Show file preview panel
- Support vim-like keybindings
- Integration with `fzf` via piping

---

### 3. Content Search ⭐⭐⭐
**Status**: TBD
**Impact**: Medium
**Complexity**: Medium

#### Proposed API
```bash
# Search file content (in addition to metadata)
fmd -t Finance --content "Apple stock"

# Regex search in content
fmd --content-regex "TODO|FIXME"

# Show matching context
fmd --content "Docker" --show-context 3
```

#### Implementation Notes
- Search within file content after metadata filtering
- Use existing `regex` crate
- Option to show surrounding lines (like `grep -C`)
- Respect `--head` limit or search full file

---

### 4. Exclusion Filters (NOT logic) ⭐⭐⭐
**Status**: TBD
**Impact**: Medium
**Complexity**: Low

#### Proposed API
```bash
# Exclude tags
fmd --exclude-tag Archive --exclude-tag Draft

# Exclude file patterns
fmd --exclude-name "private*"

# Exclude directories
fmd --exclude-dir ".git" --exclude-dir "archive"
```

#### Implementation Notes
- Add `exclude_tags`, `exclude_names`, `exclude_dirs` to Args
- Apply exclusion filters after inclusion filters
- Useful for filtering out archived/draft files

---

### 5. Configuration File ⭐⭐⭐
**Status**: TBD
**Impact**: Medium
**Complexity**: Low

#### Proposed API
```bash
# Use config file at ~/.config/fmd/config.toml
fmd  # Automatically loads config

# Override config location
fmd --config ~/custom/fmd.toml
```

#### Example Config
```toml
[defaults]
head_lines = 20
ignore_case = false

[search]
ignore_dirs = [".git", "node_modules", "target", ".obsidian"]

[aliases]
work = ["-t", "Work", "-f", "status:active"]
todos = ["-t", "TODO", "--sort", "date"]
recent = ["--modified-within", "7 days"]
```

#### Implementation Notes
- Use `toml` or `config` crate
- Load from `~/.config/fmd/config.toml`
- Command-line args override config values

---

### 6. Advanced Regex Support ⭐⭐⭐
**Status**: Future
**Impact**: Medium
**Complexity**: Low

#### Enhancements
- Regex caching for repeated patterns
- Better error messages for invalid regex
- Case-insensitive regex flag
- Multi-line regex matching

#### Example
```bash
# OR matching in title
fmd -T "Meeting|Discussion|Standup"

# Strict filename matching
fmd -n "^2025-\d{2}-\d{2}\.md$"
```

---

### 7. Output Format Options ⭐⭐
**Status**: TBD
**Impact**: Medium - Better integration with other tools
**Complexity**: Medium

#### Proposed API
```bash
# JSON output (default: newline-delimited paths)
fmd -t Work --format json

# Output with metadata
fmd -t Finance --format json-pretty

# Table format
fmd -t TODO --format table

# Custom format template
fmd -t Work --format "{path}\t{title}\t{date}"
```

#### Example Outputs

**JSON Format**:
```json
[
  {
    "path": "notes/meeting.md",
    "title": "Team Meeting",
    "tags": ["work", "meeting"],
    "date": "2025-11-04",
    "author": "John Doe"
  }
]
```

**Table Format**:
```
PATH                    TITLE            TAGS         DATE
notes/meeting.md        Team Meeting     work,meeting 2025-11-04
notes/report.md         Q4 Report        finance,work 2025-11-03
```

#### Implementation Notes
- Add `serde_json` for JSON serialization
- Add `--format` enum: `path` (default), `json`, `json-pretty`, `table`, `custom`
- For `custom`, support template variables: `{path}`, `{title}`, `{tags}`, `{date}`, etc.

---

### 8. Natural Language Query ⭐
**Status**: Experimental
**Impact**: High (if done well)
**Complexity**: Very High

```bash
# Natural language queries (using LLM or rule-based NLP)
fmd --ask "Show me work notes from last week about Docker"
```

Would require AI/ML integration or complex NLP parsing.

---

## 🛠 Technical Improvements

### Performance Optimizations
- [ ] Memory-mapped file I/O for large files
- [ ] Lazy evaluation of metadata
- [ ] SIMD-accelerated string matching
- [ ] Profile-guided optimization (PGO)

### Code Quality
- [ ] Comprehensive test suite
- [ ] Benchmarking suite
- [ ] Documentation improvements
- [ ] Error handling improvements
- [ ] Logging support (`env_logger`)

### Distribution
- [ ] Pre-built binaries for Linux, macOS, Windows
- [ ] Package for Homebrew
- [ ] Package for AUR (Arch Linux)
- [ ] Snap/Flatpak package
- [ ] Docker image

### Shell Integration
- [ ] Bash completion
- [ ] Zsh completion
- [ ] Fish completion
- [ ] PowerShell completion

# fmd Feature Roadmap

This document outlines planned features and improvements for fmd.

## Priority Legend
- ⭐⭐⭐⭐⭐ Critical / High Impact
- ⭐⭐⭐⭐ Important / Very Useful
- ⭐⭐⭐ Nice to Have
- ⭐⭐ Low Priority
- ⭐ Future / Experimental

---

## ✅ Completed Features (v0.1.0)

### Full-Text Search ⭐⭐⭐⭐
**Status**: ✅ Completed (v0.1.0)
**Impact**: High - Essential for finding tags anywhere in files

#### Implementation
- Added `--full-text` flag to scan entire file content
- By default, only scans first 10 lines (controlled by `--head`)
- Works with all filter types (-t, -T, -f, -n)

#### Usage
```bash
# Default: fast, scans first 10 lines only
fmd -t ProjectTag

# Full-text: slower, scans entire file
fmd -t ProjectTag --full-text
```

---

## 🔧 Near-Term Fixes (v0.1.x Maintenance)

These are low-risk, high-impact improvements planned for the next v0.1.x patch release before v0.2.0.

### 1. Unify Walking, Globs, and Ignores ⭐⭐⭐⭐
**Status**: Planned  
**Impact**: High — Makes `--glob` effective and skips noisy dirs  
**Complexity**: Medium

#### Implementation Notes
- Replace custom `walkdir` traversal with `ignore` crate to get fast recursive walking, `.gitignore`/global ignores, and efficient file enumeration.
- Implement `--glob` using `ignore::overrides` or `globset` so patterns like `**/*.md`, `notes/**/*.md`, or multiple `--glob` can be honored.
- Default ignore directories: `.git`, `target`, `node_modules`, `.obsidian` (and respect user `.gitignore`).

#### CLI Notes
- Keep existing `--glob GLOB` flag. Optionally allow multiple occurrences (e.g., `--glob "**/*.md" --glob "**/*.mdx"`). Backward compatible.

#### Estimated Effort
~120–180 LOC, 2–4 hours

---

### 2. Case-Insensitive Regex Correctness ⭐⭐⭐⭐
**Status**: Planned  
**Impact**: Medium-High — Correctness for `-n/--name` with regex  
**Complexity**: Low

#### Implementation Notes
- Replace “to_lowercase + Regex::new” approach with `regex::RegexBuilder` and `case_insensitive(true)` when `-i/--ignore-case` is set, avoiding semantic changes to user-provided patterns.

#### Estimated Effort
~20–40 LOC, <1 hour

---

### 3. Title and Inline Tags Scanning Robustness ⭐⭐⭐⭐
**Status**: Planned  
**Impact**: Medium-High — Fewer misses in common note styles  
**Complexity**: Low-Medium

#### Implementation Notes
- Title matching: optionally recognize headings `^#{1,6}\\s` instead of only `# `.
- Inline tags (default, non `--full-text`): within the first `--head N` lines,
  - treat leading whitespace before `tags:` as valid,
  - detect `#tag` tokens in addition to `tags:` lines, to better match common inline styles.
- Keep `--full-text` behavior unchanged (search `#tag` anywhere).

#### Estimated Effort
~40–80 LOC, 1–2 hours

---

### 4. Basic Tests and Fixtures ⭐⭐⭐⭐
**Status**: Planned  
**Impact**: High — Prevent regressions while v0.2.0 evolves  
**Complexity**: Low-Medium

#### Implementation Notes
- Add `tests/fixtures/` with samples: YAML frontmatter (single/array/multi-line), inline fields, multiple heading levels, filenames for regex/i-case edge cases, ignored directories.
- Write integration tests covering filter logic: same-type OR, cross-type AND, `--head` vs `--full-text`, `--glob`, and `.gitignore` handling.

#### Estimated Effort
~4–6 tests, 2–3 hours

---

## 🎯 Top Priority Features (Next Release - v0.2.0)

### 1. Date Filtering ⭐⭐⭐⭐⭐
**Status**: Planned
**Impact**: High - Essential for note management
**Complexity**: Medium

#### Proposed API
```bash
# Filter by date range
fmd --date-after 2025-01-01
fmd --date-before 2025-12-31
fmd --date-after 2025-01-01 --date-before 2025-03-31

# Relative dates
fmd --date-after "7 days ago"
fmd --date-after "last week"
fmd --date-after "this month"

# Modified time (using file system metadata)
fmd --modified-after 2025-01-01
fmd --modified-within "7 days"
```

#### Implementation Notes
- Add `chrono` dependency for date parsing
- Support both ISO 8601 format and natural language
- Read `date` field from YAML frontmatter
- Fall back to file modification time if no frontmatter date
- Support comparison operators: `after`, `before`, `within`

#### Estimated Effort
~100 lines of code, 2-3 hours

---

### 2. Sorting Options ⭐⭐⭐⭐
**Status**: Planned
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

#### Estimated Effort
~50 lines of code, 1-2 hours

---

### 3. Output Format Options ⭐⭐⭐⭐
**Status**: Planned
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

#### Estimated Effort
~150 lines of code, 3-4 hours

---

### 4. Statistics and Aggregation ⭐⭐⭐⭐
**Status**: Planned
**Impact**: Medium - Provides insights
**Complexity**: Medium

#### Proposed API
```bash
# Show statistics
fmd --stats

# Group by tag
fmd --stats-by tag

# Group by author
fmd --stats-by author

# Group by date (by month)
fmd --stats-by date
```

#### Example Output
```
Total files: 523

Tags:
  work: 128 files (24.5%)
  personal: 95 files (18.2%)
  finance: 73 files (14.0%)
  tech: 67 files (12.8%)

Authors:
  John Doe: 234 files (44.7%)
  Jane Smith: 156 files (29.8%)
  (unspecified): 133 files (25.4%)

Date Range: 2025-01-15 to 2025-11-04 (294 days)
```

#### Implementation Notes
- Scan all matching files
- Collect statistics in HashMap
- Sort by count (descending)
- Pretty-print results

#### Estimated Effort
~100 lines of code, 2-3 hours

---

### 5. Cache Mechanism ⭐⭐⭐⭐⭐
**Status**: Planned
**Impact**: Very High - Major performance boost
**Complexity**: High

#### Proposed API
```bash
# Build cache (first time or when files change)
fmd --build-cache

# Use cache automatically (check file mtimes)
fmd -t Work  # Uses cache if available and up-to-date

# Force rebuild cache
fmd --rebuild-cache

# Clear cache
fmd --clear-cache

# Show cache stats
fmd --cache-info
```

#### Performance Goals
- Current (no cache): 5ms for 500 files
- With cache: <1ms for 500 files
- Cache hit rate: >95%

#### Implementation Notes
- Store cache in `~/.cache/fmd/` or `~/.local/share/fmd/`
- Use SQLite or bincode for storage
- Cache format:
  ```rust
  struct CachedFile {
      path: PathBuf,
      mtime: SystemTime,
      metadata: Frontmatter,
  }
  ```
- Invalidate cache entries when file mtime changes
- Optional: watch mode using `notify` crate

#### Estimated Effort
~300 lines of code, 1-2 days

---

## 📋 Secondary Features

### 6. Interactive Mode ⭐⭐⭐⭐
**Status**: Planned
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

#### Estimated Effort
~500 lines of code, 3-5 days

---

### 7. Content Search ⭐⭐⭐
**Status**: Planned
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

#### Estimated Effort
~100 lines of code, 2-3 hours

---

### 8. Exclusion Filters (NOT logic) ⭐⭐⭐
**Status**: Planned
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

#### Estimated Effort
~50 lines of code, 1-2 hours

---

### 9. Saved Queries / Aliases ⭐⭐⭐
**Status**: Planned
**Impact**: Medium
**Complexity**: Medium

#### Proposed API
```bash
# Save a query
fmd --save-query work-todos "-t Work -f status:todo"

# List saved queries
fmd --list-queries

# Run saved query
fmd --query work-todos

# Delete saved query
fmd --delete-query work-todos
```

#### Implementation Notes
- Store queries in config file (`~/.config/fmd/queries.toml`)
- Format:
  ```toml
  [queries.work-todos]
  args = ["-t", "Work", "-f", "status:todo"]
  description = "Work items marked as TODO"
  ```

#### Estimated Effort
~100 lines of code, 2-3 hours

---

### 10. Configuration File ⭐⭐⭐
**Status**: Planned
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

[cache]
enabled = true
location = "~/.cache/fmd/"
auto_rebuild = true

[aliases]
work = ["-t", "Work", "-f", "status:active"]
todos = ["-t", "TODO", "--sort", "date"]
recent = ["--modified-within", "7 days"]
```

#### Implementation Notes
- Use `toml` or `config` crate
- Load from `~/.config/fmd/config.toml`
- Command-line args override config values

#### Estimated Effort
~100 lines of code, 2-3 hours

---

## 🔮 Future / Experimental Features

### 11. File Watch Mode ⭐⭐
**Status**: Future
**Impact**: Low
**Complexity**: Medium

```bash
# Watch for file changes and re-run query
fmd -t Work --watch
```

Uses `notify` crate to watch filesystem.

---

### 12. Batch Operations ⭐⭐
**Status**: Future (consider security implications)
**Impact**: Medium
**Complexity**: High

```bash
# Batch modify tags (DANGEROUS - needs confirmation)
fmd -t OldTag --replace-tag NewTag --dry-run
fmd -t OldTag --replace-tag NewTag --confirm

# Batch add field
fmd -t Work --add-field "reviewed:yes" --dry-run
```

⚠️ **Warning**: Modifying files is risky. Requires:
- Dry-run mode
- Confirmation prompts
- Backup mechanism
- Atomic writes

---

### 13. Git Integration ⭐⭐
**Status**: Future
**Impact**: Low
**Complexity**: Medium

```bash
# Find modified files (git status)
fmd --git-status modified

# Find files changed in last N commits
fmd --git-commits 10

# Find untracked notes
fmd --git-status untracked
```

Uses `git2` crate for Git operations.

---

### 14. Advanced Regex Support ⭐⭐⭐
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

### 15. Export/Import ⭐⭐
**Status**: Future
**Impact**: Low
**Complexity**: Medium

```bash
# Export matching files to archive
fmd -t Archive --export archive.tar.gz

# Export as CSV
fmd --format csv > notes.csv

# Import from JSON
fmd --import notes.json
```

---

### 16. Natural Language Query ⭐
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

---

## 📊 Implementation Timeline

### Phase 0: v0.1.x Maintenance
**Target**: 1–3 days
- [ ] Unify walking, globs, and ignores (ignore crate + `--glob`)
- [ ] Case-insensitive regex correctness (`RegexBuilder`)
- [ ] Title and inline tags scanning robustness
- [ ] Basic tests and fixtures

### Phase 1: Core Enhancements (v0.2.0)
**Target**: 2-3 weeks
- [x] Basic implementation (v0.1.0)
- [ ] Date filtering
- [ ] Sorting options
- [ ] Output formats (JSON, table)

### Phase 2: Performance & UX (v0.3.0)
**Target**: 1 month
- [ ] Cache mechanism
- [ ] Statistics/aggregation
- [ ] Configuration file
- [ ] Exclusion filters

### Phase 3: Advanced Features (v0.4.0)
**Target**: 2 months
- [ ] Interactive mode
- [ ] Content search
- [ ] Saved queries
- [ ] Shell completions

### Phase 4: Future (v1.0.0)
**Target**: TBD
- [ ] Git integration
- [ ] Watch mode
- [ ] Advanced features based on user feedback

---

## 🤝 Contributing

Contributions are welcome! If you'd like to implement any of these features:

1. Check this PLAN.md for status
2. Open an issue to discuss the approach
3. Submit a PR with tests and documentation
4. Update this PLAN.md with implementation notes

---

## 📝 Notes

- Features marked with ⚠️ require careful design (security, UX)
- Estimated efforts are rough guidelines
- Priority may change based on user feedback
- Some features may be dropped if complexity is too high

---

Last Updated: 2025-11-04

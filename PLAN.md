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

### 2. Content Search ⭐⭐⭐
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

### 3. Exclusion Filters (NOT logic) ⭐⭐⭐
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

### 4. Advanced Regex Support ⭐⭐⭐
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

# fmd Feature Roadmap

This document outlines planned features and improvements for fmd.

## Planned Features

### 1. Sorting Options

Sort results by various fields:
```bash
fmd -t Work --sort date         # By frontmatter date
fmd -t Work --sort title        # Alphabetically by title
fmd -t Work --sort modified     # By file modification time
fmd -t Work --sort path         # By file path
fmd -t Work --sort created      # By file creation time
fmd -t Work --sort date --reverse
```

### 2. Content Search

Search within file content in addition to metadata:
```bash
fmd -t Finance --content "Apple stock"
fmd --content-regex "TODO|FIXME"
fmd --content "Docker" --show-context 3
```

### 3. Exclusion Filters

Exclude specific items from results:
```bash
fmd --exclude-tag Archive --exclude-tag Draft
fmd --exclude-name "private*"
fmd --exclude-dir ".git" --exclude-dir "archive"
```

### 4. Advanced Regex Support

- Regex caching for repeated patterns
- Better error messages for invalid regex
- Case-insensitive regex flag
- Multi-line regex matching

## Technical Improvements

### Performance
- Memory-mapped file I/O for large files
- Lazy evaluation of metadata
- SIMD-accelerated string matching
- Profile-guided optimization (PGO)

### Distribution
- Package for Homebrew
- Debian/Ubuntu package (.deb)
- Snap/Flatpak package
- Docker image

### Shell Integration
- Zsh completion
- Bash completion
- PowerShell completion

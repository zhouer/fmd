# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of fmd (Find Markdown by metadata)
- Search Markdown files by YAML frontmatter tags
- Search by inline tags (#tag format)
- Filter by title (YAML frontmatter or Markdown headings)
- Filter by filename with regex support
- Filter by custom frontmatter fields
- Date range filtering (--date-after, --date-before)
- Support for multiple date fields (date, created, updated, modified)
- Full-text search mode (--full-text)
- Configurable metadata scan depth (--head)
- Case-insensitive filename matching (-i)
- NUL-delimited output for safe filename handling (-0)
- Depth-limited search (-d)
- Parallel file processing for performance
- Respects .gitignore and .ignore files
- Comprehensive test suite (28 unit tests + 16 integration tests)
- GitHub Actions CI/CD workflows
  - Automated testing on Linux, macOS, and Windows
  - Code quality checks (clippy, rustfmt)
  - Code coverage reporting
  - Cross-platform release builds

### Filter Logic
- OR logic within same filter type (e.g., -t python -t rust)
- AND logic across different filter types (e.g., -t work -T meeting)

### Supported Metadata Formats
- YAML frontmatter with --- delimiters
- Inline format (key: value)
- Both array and single-value YAML tags
- Multi-line YAML tag arrays

### Performance Optimizations
- Parallel file processing with rayon
- Optimized release builds (LTO, strip symbols)
- Efficient file reading (only scans necessary lines)
- Pre-compiled regex patterns
- Smart directory exclusions (target/, node_modules/, etc.)

[Unreleased]: https://github.com/zhouer/fmd/compare/HEAD...HEAD

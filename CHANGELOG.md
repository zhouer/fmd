# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-06

Initial release of **fmd** - Find Markdown files by metadata.

### Features

- **Tag filtering** (`-t`, `--tag`): Search by tags in YAML frontmatter (`tags: [tag1, tag2]`) and inline format (`#tag`)
  - Supports multiple tags with OR logic
  - Case-insensitive matching
  - Word boundary detection to prevent false matches

- **Title filtering** (`-T`, `--title`): Search by document title
  - Matches YAML frontmatter `title:` field
  - Matches Markdown headings (H1-H6)
  - Supports regex patterns

- **Author filtering** (`-a`, `--author`): Filter by author field
  - Matches YAML frontmatter `author:` field
  - Matches inline `author: value` format
  - Supports multiple authors with OR logic
  - Partial matching and case-insensitive

- **Filename filtering** (`-n`, `--name`): Filter by filename
  - Regex pattern support
  - Multiple patterns with OR logic
  - Optional case-insensitive matching (`-i`)

- **Custom field filtering** (`-f`, `--field`): Filter by any YAML frontmatter field
  - Format: `field:pattern`
  - Supports both YAML frontmatter and inline formats
  - Multiple fields with OR logic

- **Date range filtering**: Filter by date metadata
  - `--date-after YYYY-MM-DD`: Files on or after the specified date
  - `--date-before YYYY-MM-DD`: Files on or before the specified date
  - Supports multiple date fields: `date`, `created`, `updated`, `modified`
  - Can be combined for date range queries

- **Filter logic**:
  - OR logic: Multiple filters of same type (e.g., `-t python -t rust` matches files with python OR rust)
  - AND logic: Filters across different types (e.g., `-t work -T meeting` matches files with work tag AND meeting in title)

- **Glob pattern matching** (`--glob`): Customize file search patterns
  - Default: `**/*.md` (all Markdown files)
  - Supports any glob pattern (e.g., `**/*.txt`, `notes/**/*.md`)

- **Search depth control** (`-d`, `--depth`): Limit directory traversal depth

- **Metadata scan control** (`--head`): Configure how many lines to scan for metadata
  - Default: 10 lines
  - Balances performance and completeness

- **Full-text mode** (`--full-text`): Scan entire file content instead of just the first N lines

- **Case-insensitive filename matching** (`-i`, `--ignore-case`)

- **NUL-delimited output** (`-0`): Output file paths separated by NUL bytes
  - Safe for filenames with spaces and special characters
  - Perfect for piping to `xargs -0`

- **Verbose mode** (`-v`, `--verbose`): Display detailed output and warnings

- **Multiple directory support**: Search in multiple directories simultaneously

- **Parallel file processing**: Utilizes multiple CPU cores for fast searching

- **Smart directory exclusions**: Automatically skips 64 common directories
  - Build outputs: `target/`, `build/`, `dist/`, `out/`, `bin/`, `obj/`
  - Dependencies: `node_modules/`, `vendor/`, `bower_components/`, `__pycache__/`
  - Caches: `.cache/`, `.gradle/`, `.m2/`, `.parcel-cache/`
  - Frontend frameworks: `.next/`, `.nuxt/`, `.vitepress/`, `.docusaurus/`
  - IDE folders: `.idea/`, `.vscode/`, `.vs/`, `.obsidian/`

- **Git integration**: Automatically respects `.gitignore` and `.ignore` files

- **Efficient file reading**: Only reads necessary content to minimize I/O
  - Automatically detects frontmatter boundaries
  - Stops reading after metadata is extracted (unless in full-text mode)

- **Pre-compiled filters**: Regex patterns and filters compiled once at startup for optimal performance

- **Comprehensive error handling**: Clear error messages with context

- **Unix tool integration**: Seamless integration with `xargs`, `grep`, `fzf`, and other Unix tools

[0.1.0]: https://github.com/zhouer/fmd/releases/tag/v0.1.0

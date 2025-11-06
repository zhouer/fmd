use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Parser;
use globset::Glob;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Default number of lines to scan for metadata when not in full-text mode.
/// This is enough to capture typical frontmatter (usually < 10 lines) plus
/// a few lines of content for inline metadata detection.
const DEFAULT_HEAD_LINES: usize = 10;

/// Maximum number of lines to read for frontmatter to prevent memory issues.
/// Frontmatter exceeding this limit will cause the file to be skipped with an error.
const MAX_FRONTMATTER_LINES: usize = 1000;

/// Directories to always skip during file enumeration.
/// These are common build artifacts, dependencies, caches, and tool-specific directories.
const EXCLUDED_DIRS: &[&str] = &[
    // Build artifacts
    "target",
    "build",
    "dist",
    "out",
    "bin",
    "obj",
    // Dependencies
    "node_modules",
    "vendor",
    "bower_components",
    // Python
    "__pycache__",
    // Caches
    ".cache",
    ".parcel-cache",
    ".gradle",
    ".m2",
    // Frontend frameworks
    ".next",
    ".nuxt",
    ".vitepress",
    ".docusaurus",
    ".output",
    ".serverless",
    // IDEs and editors
    ".idea",
    ".vscode",
    ".vs",
    ".obsidian",
    // Temporary and test coverage
    "tmp",
    "temp",
    "coverage",
    ".nyc_output",
    ".pytest_cache",
    ".tox",
];

/// fmd — Find Markdown files by metadata
#[derive(Parser, Debug)]
#[command(name = "fmd")]
#[command(about = "Find Markdown files by metadata - Search by tags, frontmatter, and custom fields", long_about = None)]
struct Args {
    /// Use NUL-delimited output (safe for xargs -0)
    #[arg(short = '0', long)]
    nul: bool,

    /// Case-insensitive filename matching
    #[arg(short = 'i', long = "ignore-case")]
    ignore_case: bool,

    /// Limit search depth (1=current dir only, default: unlimited)
    #[arg(short = 'd', long = "depth")]
    depth: Option<usize>,

    /// Filter by tag (can be specified multiple times, OR logic)
    #[arg(short = 't', long = "tag")]
    tags: Vec<String>,

    /// Filter by title (can be specified multiple times, OR logic)
    #[arg(short = 'T', long = "title")]
    titles: Vec<String>,

    /// Filter by author (can be specified multiple times, OR logic)
    #[arg(short = 'a', long = "author")]
    authors: Vec<String>,

    /// Filter by filename (can be specified multiple times, OR logic)
    #[arg(short = 'n', long = "name")]
    names: Vec<String>,

    /// Filter by frontmatter field (format: "field:pattern", OR logic)
    #[arg(short = 'f', long = "field")]
    fields: Vec<String>,

    /// File pattern to match
    #[arg(long = "glob", default_value = "**/*.md")]
    glob: String,

    /// Lines to scan for metadata
    #[arg(long = "head", default_value_t = DEFAULT_HEAD_LINES)]
    head_lines: usize,

    /// Search full file content (not just first N lines)
    #[arg(long = "full-text")]
    full_text: bool,

    /// Show verbose output including warnings and errors
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Filter files with dates after this date (format: YYYY-MM-DD)
    #[arg(long = "date-after")]
    date_after: Option<String>,

    /// Filter files with dates before this date (format: YYYY-MM-DD)
    #[arg(long = "date-before")]
    date_before: Option<String>,

    /// Directories to search (default: current directory)
    #[arg(default_value = ".")]
    dirs: Vec<PathBuf>,
}

/// Pre-compiled filters for efficient matching
struct CompiledFilters {
    /// Tag patterns: (lowercase_pattern, regex) for matching both YAML and inline tags
    tag_patterns: Vec<(String, Regex)>,

    /// Pre-lowercased title patterns for case-insensitive matching
    title_patterns: Vec<String>,

    /// Pre-lowercased author patterns for case-insensitive matching
    author_patterns: Vec<String>,

    /// Pre-compiled regex patterns for filename matching
    name_patterns: Vec<Regex>,

    /// Pre-parsed field filters (field_name, pattern_lowercase)
    field_patterns: Vec<(String, String)>,

    /// Date filter: files with dates on or after this date
    date_after: Option<NaiveDate>,

    /// Date filter: files with dates on or before this date
    date_before: Option<NaiveDate>,
}

impl CompiledFilters {
    fn from_args(args: &Args) -> Result<Self> {
        // Compile tag regex patterns
        let mut tag_patterns = Vec::new();
        for tag in &args.tags {
            let pattern = tag.strip_prefix('#').unwrap_or(tag);
            // Use word boundaries instead of lookbehind/lookahead (not supported in Rust regex)
            // Match #tag with optional surrounding non-word characters
            let regex = RegexBuilder::new(&format!(
                r"(^|[^[:word:]])#{}([^[:word:]]|$)",
                regex::escape(pattern)
            ))
            .case_insensitive(true)
            .build()
            .with_context(|| format!("Failed to compile tag pattern: {}", tag))?;
            tag_patterns.push((pattern.to_lowercase(), regex));
        }

        // Pre-lowercase title patterns
        let title_patterns = args.titles.iter().map(|t| t.to_lowercase()).collect();

        // Pre-lowercase author patterns
        let author_patterns = args.authors.iter().map(|a| a.to_lowercase()).collect();

        // Compile filename regex patterns
        let mut name_patterns = Vec::new();
        for name in &args.names {
            let regex = RegexBuilder::new(name)
                .case_insensitive(args.ignore_case)
                .build()
                .with_context(|| format!("Failed to compile filename pattern: {}", name))?;
            name_patterns.push(regex);
        }

        // Parse field filters
        let mut field_patterns = Vec::new();
        for field_spec in &args.fields {
            let (field, pattern) = field_spec.split_once(':').ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid field filter format: '{}'. Expected 'field:pattern'",
                    field_spec
                )
            })?;

            let field_trimmed = field.trim();
            let pattern_trimmed = pattern.trim();

            // Validate that both field and pattern are non-empty
            if field_trimmed.is_empty() && pattern_trimmed.is_empty() {
                return Err(anyhow::anyhow!(
                    "Both field and pattern cannot be empty in filter '{}'",
                    field_spec
                ));
            }
            if field_trimmed.is_empty() {
                return Err(anyhow::anyhow!(
                    "Field name cannot be empty in filter '{}'",
                    field_spec
                ));
            }
            if pattern_trimmed.is_empty() {
                return Err(anyhow::anyhow!(
                    "Pattern cannot be empty in filter '{}'",
                    field_spec
                ));
            }

            field_patterns.push((field_trimmed.to_string(), pattern_trimmed.to_lowercase()));
        }

        // Parse date filters
        let date_after = if let Some(date_str) = &args.date_after {
            Some(
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").with_context(|| {
                    format!(
                        "Invalid date format for --date-after: '{}'. Expected YYYY-MM-DD",
                        date_str
                    )
                })?,
            )
        } else {
            None
        };

        let date_before = if let Some(date_str) = &args.date_before {
            Some(
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").with_context(|| {
                    format!(
                        "Invalid date format for --date-before: '{}'. Expected YYYY-MM-DD",
                        date_str
                    )
                })?,
            )
        } else {
            None
        };

        Ok(CompiledFilters {
            tag_patterns,
            title_patterns,
            author_patterns,
            name_patterns,
            field_patterns,
            date_after,
            date_before,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Frontmatter {
    #[serde(default)]
    title: Option<String>,

    #[serde(default)]
    author: Option<String>,

    #[serde(default)]
    tags: Option<TagValue>,

    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum TagValue {
    Single(String),
    Array(Vec<String>),
}

impl TagValue {
    fn contains_tag(&self, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        match self {
            TagValue::Single(tag) => tag.to_lowercase().contains(&pattern_lower),
            TagValue::Array(tags) => tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&pattern_lower)),
        }
    }
}

/// Helper function to match a pattern against various YAML value types (case-insensitive)
fn yaml_value_contains(value: &serde_yaml::Value, pattern_lower: &str) -> bool {
    match value {
        serde_yaml::Value::String(s) => s.to_lowercase().contains(pattern_lower),
        serde_yaml::Value::Number(n) => n.to_string().to_lowercase().contains(pattern_lower),
        serde_yaml::Value::Bool(b) => b.to_string().to_lowercase().contains(pattern_lower),
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().any(|v| yaml_value_contains(v, pattern_lower))
        }
        _ => false,
    }
}

/// Helper function to parse a date from a YAML value
fn parse_date_from_yaml_value(value: &serde_yaml::Value) -> Option<NaiveDate> {
    match value {
        serde_yaml::Value::String(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").ok(),
        _ => None,
    }
}

/// Metadata extracted from a markdown file.
///
/// Design note: This struct stores `raw_content` as a String, which may seem memory-intensive.
/// However, this design is intentional and already optimized:
///
/// - In default mode: Only first N lines are read (using BufReader), typically ~10 lines
/// - In full_text mode: Entire file is read, but this is required for the search
/// - Memory is released immediately after filtering (not stored long-term)
/// - Parallel processing (rayon) limits concurrent file reads to available CPU threads
///
/// Typical memory usage: ~10KB per file in default mode, ~100KB in full-text mode.
/// Peak memory with 8 threads: ~800KB (8 files × 100KB), which is acceptable.
struct Metadata {
    frontmatter: Option<Frontmatter>,
    raw_content: String,
}

impl Metadata {
    fn from_file(path: &Path, head_lines: usize, full_text: bool, _verbose: bool) -> Result<Self> {
        // Read file content efficiently (only what we need)
        let content = read_file_content(path, head_lines, full_text)?;

        // Try to extract YAML frontmatter
        let frontmatter = extract_frontmatter(&content, path);

        // The content we read is already optimized for the mode
        Ok(Metadata {
            frontmatter,
            raw_content: content,
        })
    }

    fn has_tag(&self, pattern_lower: &str, tag_regex: &Regex) -> bool {
        // Check YAML frontmatter (case-insensitive)
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref tags) = fm.tags {
                if tags.contains_tag(pattern_lower) {
                    return true;
                }
            }
        }

        // Check inline tags with regex (case-insensitive, works for both full_text and default mode)
        tag_regex.is_match(&self.raw_content)
    }

    fn has_title(&self, pattern_lower: &str) -> bool {
        // Check YAML frontmatter title
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref title) = fm.title {
                if title.to_lowercase().contains(pattern_lower) {
                    return true;
                }
            }
        }

        // Check markdown headings (levels 1–6), allow leading whitespace
        const MAX_HEADING_LEVEL: usize = 6;
        for line in self.raw_content.lines() {
            let trimmed = line.trim_start();
            // Count leading '#'
            let mut hashes = 0;
            for ch in trimmed.chars() {
                if ch == '#' {
                    hashes += 1;
                } else {
                    break;
                }
            }
            if (1..=MAX_HEADING_LEVEL).contains(&hashes) {
                // Expect a space after the hashes
                let after = &trimmed[hashes..];
                if after.starts_with(' ') && after.to_lowercase().contains(pattern_lower) {
                    return true;
                }
            }
        }

        false
    }

    fn has_author(&self, pattern_lower: &str) -> bool {
        // Check YAML frontmatter author
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref author) = fm.author {
                if author.to_lowercase().contains(pattern_lower) {
                    return true;
                }
            }
        }

        // Check inline format (author: value)
        for line in self.raw_content.lines() {
            let trimmed = line.trim_start();
            if let Some(colon_pos) = trimmed.find(':') {
                let key = &trimmed[..colon_pos];
                if key.eq_ignore_ascii_case("author") {
                    let value = &trimmed[colon_pos + 1..];
                    if value.to_lowercase().contains(pattern_lower) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn has_field(&self, field_name: &str, pattern_lower: &str) -> bool {
        // Check YAML frontmatter
        if let Some(ref fm) = self.frontmatter {
            if let Some(value) = fm.extra.get(field_name) {
                if yaml_value_contains(value, pattern_lower) {
                    return true;
                }
            }
        }

        // Check simple inline format (key: value)
        // Only search in the value part, not the key
        for line in self.raw_content.lines() {
            let trimmed = line.trim_start();
            if let Some(colon_pos) = trimmed.find(':') {
                let key = &trimmed[..colon_pos];
                if key.eq_ignore_ascii_case(field_name) {
                    let value = &trimmed[colon_pos + 1..];
                    if value.to_lowercase().contains(pattern_lower) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Extract dates from the frontmatter or content.
    /// Checks for date, created, updated, modified fields.
    /// Returns a list of all valid dates found (deduplicated).
    fn extract_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let date_fields = ["date", "created", "updated", "modified"];

        // Check YAML frontmatter first
        if let Some(ref fm) = self.frontmatter {
            for field_name in &date_fields {
                if let Some(value) = fm.extra.get(*field_name) {
                    if let Some(date) = parse_date_from_yaml_value(value) {
                        dates.push(date);
                    }
                }
            }
        } else {
            // Only check inline format if no frontmatter exists
            // to avoid duplicates
            for line in self.raw_content.lines() {
                let trimmed = line.trim_start();
                if let Some(colon_pos) = trimmed.find(':') {
                    let key = &trimmed[..colon_pos].trim();
                    if date_fields.iter().any(|f| key.eq_ignore_ascii_case(f)) {
                        let value = &trimmed[colon_pos + 1..].trim();
                        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                            dates.push(date);
                        }
                    }
                }
            }
        }

        // Deduplicate dates
        dates.sort();
        dates.dedup();
        dates
    }

    /// Check if any date matches the date filters
    fn matches_date_filters(
        &self,
        date_after: Option<NaiveDate>,
        date_before: Option<NaiveDate>,
    ) -> bool {
        let dates = self.extract_dates();

        // If no dates found, don't match date filters
        if dates.is_empty() {
            return false;
        }

        // Check if ANY date satisfies the filters
        dates.iter().any(|date| {
            let after_check = date_after.is_none_or(|after| date >= &after);
            let before_check = date_before.is_none_or(|before| date <= &before);
            after_check && before_check
        })
    }
}

/// Read file content efficiently based on mode
/// - If full_text: read entire file
/// - If not full_text: read only first N lines (or until frontmatter end, whichever is longer)
fn read_file_content(path: &Path, head_lines: usize, full_text: bool) -> Result<String> {
    if full_text {
        // Read entire file
        return fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()));
    }

    // Open file with buffered reader for efficient line-by-line reading
    let file =
        fs::File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut lines_vec = Vec::new();
    let mut line_count = 0;
    let mut in_frontmatter = false;
    let mut frontmatter_ended = false;

    for line_result in reader.lines() {
        let line = line_result
            .with_context(|| format!("Failed to read line from file: {}", path.display()))?;

        // Track frontmatter boundaries
        let trimmed = line.trim();
        if line_count == 0 && trimmed == "---" {
            in_frontmatter = true;
        } else if in_frontmatter && trimmed == "---" {
            in_frontmatter = false;
            frontmatter_ended = true;
        }

        lines_vec.push(line);
        line_count += 1;

        // Check for excessively large frontmatter
        if in_frontmatter && line_count > MAX_FRONTMATTER_LINES {
            return Err(anyhow::anyhow!(
                "Frontmatter exceeds maximum size ({} lines) in: {}",
                MAX_FRONTMATTER_LINES,
                path.display()
            ));
        }

        // Stop reading if:
        // 1. We've read enough lines AND
        // 2. We're not in the middle of frontmatter
        if line_count >= head_lines && (!in_frontmatter || frontmatter_ended) {
            break;
        }
    }

    Ok(lines_vec.join("\n"))
}

/// Extracts YAML frontmatter from markdown content.
///
/// Frontmatter must be delimited by `---` at the start and end.
/// Returns `None` if no valid frontmatter is found or if YAML parsing fails.
/// YAML parsing errors are always logged to stderr as they affect search accuracy.
fn extract_frontmatter(content: &str, path: &Path) -> Option<Frontmatter> {
    let mut lines = content.lines();

    // Check if first line is "---"
    if lines.next()?.trim() != "---" {
        return None;
    }

    // Collect lines until next "---"
    let mut yaml_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        yaml_lines.push(line);
    }

    if yaml_lines.is_empty() {
        return None;
    }

    let yaml_content = yaml_lines.join("\n");
    match serde_yaml::from_str(&yaml_content) {
        Ok(fm) => Some(fm),
        Err(e) => {
            eprintln!(
                "Warning: Failed to parse YAML frontmatter in {}: {}",
                path.display(),
                e
            );
            None
        }
    }
}

/// Checks if a file path's filename matches a given regex pattern.
fn matches_filename(path: &Path, regex: &Regex) -> bool {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    regex.is_match(filename)
}

/// Determines if a file should be included based on its content metadata.
///
/// Applies all content-based filters (tags, titles, fields, dates) with AND logic between filter types
/// and OR logic within each filter type (e.g., match any of the specified tags).
fn should_include_file_by_content(metadata: &Metadata, filters: &CompiledFilters) -> bool {
    // Check tag filters (OR logic: match any tag)
    if !filters.tag_patterns.is_empty() {
        let tag_matched = filters
            .tag_patterns
            .iter()
            .any(|(pattern, regex)| metadata.has_tag(pattern, regex));
        if !tag_matched {
            return false;
        }
    }

    // Check title filters
    if !filters.title_patterns.is_empty() {
        let title_matched = filters
            .title_patterns
            .iter()
            .any(|pattern| metadata.has_title(pattern));
        if !title_matched {
            return false;
        }
    }

    // Check author filters (OR logic: match any author)
    if !filters.author_patterns.is_empty() {
        let author_matched = filters
            .author_patterns
            .iter()
            .any(|pattern| metadata.has_author(pattern));
        if !author_matched {
            return false;
        }
    }

    // Check field filters
    if !filters.field_patterns.is_empty() {
        let field_matched = filters
            .field_patterns
            .iter()
            .any(|(field, pattern)| metadata.has_field(field, pattern));
        if !field_matched {
            return false;
        }
    }

    // Check date filters (if any date filter is specified)
    if (filters.date_after.is_some() || filters.date_before.is_some())
        && !metadata.matches_date_filters(filters.date_after, filters.date_before)
    {
        return false;
    }

    true
}

/// Outputs file paths to stdout, either newline-delimited or NUL-delimited.
fn output_files(files: &[PathBuf], use_nul: bool) {
    for file in files {
        if use_nul {
            print!("{}\0", file.display());
        } else {
            println!("{}", file.display());
        }
    }
}

/// Enumerates all files matching the glob pattern in the specified directories.
///
/// Respects .gitignore, .ignore files, and skips hidden files and common build/cache directories.
fn enumerate_files(args: &Args) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Build glob matcher from the glob pattern
    let glob_matcher = Glob::new(&args.glob)
        .with_context(|| format!("Invalid glob pattern: {}", args.glob))?
        .compile_matcher();

    for dir in &args.dirs {
        // Use ignore crate's WalkBuilder for better performance and .gitignore support
        let mut walker = WalkBuilder::new(dir);

        // Respect .gitignore files
        walker.git_ignore(true);

        // Respect global gitignore
        walker.git_global(true);

        // Respect .ignore files
        walker.ignore(true);

        // Filter hidden files/directories (like .git, .obsidian)
        walker.hidden(true);

        // Don't follow symbolic links to avoid infinite loops
        walker.follow_links(false);

        // Set max depth if specified
        if let Some(depth) = args.depth {
            walker.max_depth(Some(depth));
        }

        for entry in walker.build().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Additional filtering for specific directories we always want to skip
            // (in case they're not hidden or not in .gitignore)
            let should_skip = path.components().any(|component| {
                if let std::path::Component::Normal(os_str) = component {
                    if let Some(dir_name) = os_str.to_str() {
                        EXCLUDED_DIRS.contains(&dir_name)
                    } else {
                        false
                    }
                } else {
                    false
                }
            });

            if should_skip {
                continue;
            }

            // Check if it's a file and matches the glob pattern
            // Match against the full path to support patterns like "**/*.md"
            // as well as simple filename patterns like "*.md"
            if path.is_file() && glob_matcher.is_match(path) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Core logic for finding matching markdown files based on filters
fn find_matching_files(args: &Args) -> Result<Vec<PathBuf>> {
    // Enumerate all markdown files
    let mut files = enumerate_files(args)?;

    // If no filters, return all files sorted
    if args.tags.is_empty()
        && args.titles.is_empty()
        && args.authors.is_empty()
        && args.names.is_empty()
        && args.fields.is_empty()
        && args.date_after.is_none()
        && args.date_before.is_none()
    {
        files.sort();
        return Ok(files);
    }

    // Compile filters once before parallel processing
    let filters = CompiledFilters::from_args(args)?;

    // Early filtering: check filename patterns first (no I/O required)
    if !filters.name_patterns.is_empty() {
        files.retain(|path| {
            filters
                .name_patterns
                .iter()
                .any(|regex| matches_filename(path, regex))
        });
    }

    // Filter files in parallel (only for content-based filters)
    let verbose = args.verbose;
    let head_lines = args.head_lines;
    let full_text = args.full_text;
    let mut matching_files: Vec<PathBuf> = files
        .par_iter()
        .filter_map(|path| {
            // Extract metadata (only if we need to check content-based filters)
            match Metadata::from_file(path, head_lines, full_text, verbose) {
                Ok(metadata) => {
                    // Check content-based filters
                    if should_include_file_by_content(&metadata, &filters) {
                        Some(path.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                    }
                    None
                }
            }
        })
        .collect();

    // Sort results alphabetically (like ls)
    matching_files.sort();

    Ok(matching_files)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let matching_files = find_matching_files(&args)?;
    output_files(&matching_files, args.nul);
    Ok(())
}

#[cfg(test)]
mod tests;

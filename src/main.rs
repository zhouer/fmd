use anyhow::{Context, Result};
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

    /// Directories to search (default: current directory)
    #[arg(default_value = ".")]
    dirs: Vec<PathBuf>,
}

/// Pre-compiled filters for efficient matching
struct CompiledFilters {
    /// Pre-compiled regex patterns for tags (with lowercase patterns for reference)
    tag_patterns: Vec<(String, Regex)>,

    /// Pre-lowercased title patterns for case-insensitive matching
    title_patterns: Vec<String>,

    /// Pre-compiled regex patterns for filename matching
    name_patterns: Vec<Regex>,

    /// Pre-parsed field filters (field_name, pattern_lowercase)
    field_patterns: Vec<(String, String)>,
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
        let title_patterns = args.titles.iter()
            .map(|t| t.to_lowercase())
            .collect();

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
            if let Some((field, pattern)) = field_spec.split_once(':') {
                field_patterns.push((
                    field.trim().to_string(),
                    pattern.trim().to_lowercase()
                ));
            }
        }

        Ok(CompiledFilters {
            tag_patterns,
            title_patterns,
            name_patterns,
            field_patterns,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Frontmatter {
    #[serde(default)]
    title: Option<String>,

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
            TagValue::Array(tags) => tags.iter().any(|tag| tag.to_lowercase().contains(&pattern_lower)),
        }
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
    fn from_file(path: &Path, head_lines: usize, full_text: bool, verbose: bool) -> Result<Self> {
        // Read file content efficiently (only what we need)
        let content = read_file_content(path, head_lines, full_text)?;

        // Try to extract YAML frontmatter
        let frontmatter = extract_frontmatter(&content, path, verbose);

        // The content we read is already optimized for the mode
        Ok(Metadata {
            frontmatter,
            raw_content: content,
        })
    }

    fn has_tag(&self, pattern_lower: &str, tag_regex: &Regex) -> bool {
        // Check YAML frontmatter
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref tags) = fm.tags {
                if tags.contains_tag(pattern_lower) {
                    return true;
                }
            }
        }

        // Check inline tags with regex (works for both full_text and default mode)
        // This provides consistent behavior across both modes
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
                if ch == '#' { hashes += 1; } else { break; }
            }
            if hashes >= 1 && hashes <= MAX_HEADING_LEVEL {
                // Expect a space after the hashes
                let after = &trimmed[hashes..];
                if after.starts_with(' ') {
                    if after.to_lowercase().contains(pattern_lower) {
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
                match value {
                    serde_yaml::Value::String(s) => {
                        if s.to_lowercase().contains(pattern_lower) {
                            return true;
                        }
                    }
                    serde_yaml::Value::Number(n) => {
                        if n.to_string().to_lowercase().contains(pattern_lower) {
                            return true;
                        }
                    }
                    serde_yaml::Value::Bool(b) => {
                        if b.to_string().to_lowercase().contains(pattern_lower) {
                            return true;
                        }
                    }
                    serde_yaml::Value::Sequence(seq) => {
                        if seq.iter().any(|v| {
                            if let serde_yaml::Value::String(s) = v {
                                s.to_lowercase().contains(pattern_lower)
                            } else {
                                false
                            }
                        }) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check simple inline format
        let field_prefix = format!("{}:", field_name).to_lowercase();
        for line in self.raw_content.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.starts_with(&field_prefix) {
                if line_lower.contains(pattern_lower) {
                    return true;
                }
            }
        }

        false
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
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
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

        // Stop reading if:
        // 1. We've read enough lines AND
        // 2. We're not in the middle of frontmatter
        if line_count >= head_lines && (!in_frontmatter || frontmatter_ended) {
            break;
        }
    }

    Ok(lines_vec.join("\n"))
}

fn extract_frontmatter(content: &str, path: &Path, verbose: bool) -> Option<Frontmatter> {
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
            if verbose {
                eprintln!("Warning: Failed to parse YAML frontmatter in {}: {}", path.display(), e);
            }
            None
        }
    }
}

fn matches_filename(path: &Path, regex: &Regex) -> bool {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    regex.is_match(filename)
}

fn should_include_file_by_content(
    metadata: &Metadata,
    filters: &CompiledFilters,
) -> bool {
    // Check tag filters
    if !filters.tag_patterns.is_empty() {
        let tag_matched = filters.tag_patterns.iter().any(|(pattern, regex)| {
            metadata.has_tag(pattern, regex)
        });
        if !tag_matched {
            return false;
        }
    }

    // Check title filters
    if !filters.title_patterns.is_empty() {
        let title_matched = filters.title_patterns.iter().any(|pattern| {
            metadata.has_title(pattern)
        });
        if !title_matched {
            return false;
        }
    }

    // Check field filters
    if !filters.field_patterns.is_empty() {
        let field_matched = filters.field_patterns.iter().any(|(field, pattern)| {
            metadata.has_field(field, pattern)
        });
        if !field_matched {
            return false;
        }
    }

    true
}

fn output_files(files: &[PathBuf], use_nul: bool) {
    for file in files {
        if use_nul {
            print!("{}\0", file.display());
        } else {
            println!("{}", file.display());
        }
    }
}

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
            // Use proper path component checking instead of string matching
            let should_skip = path.components().any(|component| {
                if let std::path::Component::Normal(os_str) = component {
                    matches!(
                        os_str.to_str(),
                        // Build artifacts
                        Some("target") | Some("build") | Some("dist") | Some("out") |
                        Some("bin") | Some("obj") |
                        // Dependencies
                        Some("node_modules") | Some("vendor") | Some("bower_components") |
                        // Python
                        Some("__pycache__") |
                        // Caches
                        Some(".cache") | Some(".parcel-cache") | Some(".gradle") | Some(".m2") |
                        // Frontend frameworks
                        Some(".next") | Some(".nuxt") | Some(".vitepress") | Some(".docusaurus") |
                        Some(".output") | Some(".serverless") |
                        // IDEs and editors
                        Some(".idea") | Some(".vscode") | Some(".vs") | Some(".obsidian") |
                        // Temporary and test coverage
                        Some("tmp") | Some("temp") | Some("coverage") | Some(".nyc_output") |
                        Some(".pytest_cache") | Some(".tox")
                    )
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

fn main() -> Result<()> {
    let args = Args::parse();

    // Enumerate all markdown files
    let mut files = enumerate_files(&args)?;

    // If no filters, just output all files
    if args.tags.is_empty()
        && args.titles.is_empty()
        && args.names.is_empty()
        && args.fields.is_empty()
    {
        // Sort results alphabetically (like ls)
        files.sort();
        output_files(&files, args.nul);
        return Ok(());
    }

    // Compile filters once before parallel processing
    let filters = CompiledFilters::from_args(&args)?;

    // Early filtering: check filename patterns first (no I/O required)
    if !filters.name_patterns.is_empty() {
        files.retain(|path| {
            filters.name_patterns.iter().any(|regex| {
                matches_filename(path, regex)
            })
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

    // Output results
    output_files(&matching_files, args.nul);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter_valid() {
        let content = "---\ntitle: Test\ntags: [rust, cli]\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);

        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.title, Some("Test".to_string()));
    }

    #[test]
    fn test_extract_frontmatter_empty() {
        let content = "---\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);

        assert!(fm.is_none());
    }

    #[test]
    fn test_extract_frontmatter_none() {
        let content = "# Just a heading";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);

        assert!(fm.is_none());
    }

    #[test]
    fn test_extract_frontmatter_multiline_tags() {
        let content = "---\ntags:\n  - rust\n  - cli\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);

        assert!(fm.is_some());
        let fm = fm.unwrap();
        if let Some(TagValue::Array(tags)) = fm.tags {
            assert_eq!(tags.len(), 2);
            assert!(tags.contains(&"rust".to_string()));
            assert!(tags.contains(&"cli".to_string()));
        } else {
            panic!("Expected array of tags");
        }
    }

    #[test]
    fn test_tag_value_contains_tag() {
        let single = TagValue::Single("rust".to_string());
        assert!(single.contains_tag("rust"));
        assert!(single.contains_tag("RUST")); // case insensitive
        assert!(!single.contains_tag("python"));

        let array = TagValue::Array(vec!["rust".to_string(), "cli".to_string()]);
        assert!(array.contains_tag("rust"));
        assert!(array.contains_tag("CLI")); // case insensitive
        assert!(!array.contains_tag("python"));
    }

    #[test]
    fn test_compiled_filters_tag_regex() {
        let args = Args {
            tags: vec!["rust".to_string(), "python".to_string()],
            titles: vec![],
            names: vec![],
            fields: vec![],
            nul: false,
            ignore_case: false,
            depth: None,
            glob: "**/*.md".to_string(),
            head_lines: 10,
            full_text: false,
            verbose: false,
            dirs: vec![PathBuf::from(".")],
        };

        let filters = CompiledFilters::from_args(&args).unwrap();
        assert_eq!(filters.tag_patterns.len(), 2);

        // Check that regex matches work
        let (pattern, regex) = &filters.tag_patterns[0];
        assert_eq!(pattern, "rust");
        assert!(regex.is_match("#rust"));
        assert!(regex.is_match("#RUST")); // case insensitive
        assert!(!regex.is_match("#rust123")); // word boundary
    }

    #[test]
    fn test_compiled_filters_invalid_regex_returns_error() {
        let args = Args {
            tags: vec![],
            titles: vec![],
            names: vec!["[invalid".to_string()], // Invalid regex
            fields: vec![],
            nul: false,
            ignore_case: false,
            depth: None,
            glob: "**/*.md".to_string(),
            head_lines: 10,
            full_text: false,
            verbose: false,
            dirs: vec![PathBuf::from(".")],
        };

        let result = CompiledFilters::from_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_has_tag_yaml() {
        let content = "---\ntags: [rust, cli]\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);
        let metadata = Metadata {
            frontmatter: fm,
            raw_content: content.to_string(),
        };

        let (pattern, regex) = ("rust".to_string(),
            regex::RegexBuilder::new(r"(^|[^[:word:]])#rust([^[:word:]]|$)")
                .case_insensitive(true)
                .build()
                .unwrap());

        assert!(metadata.has_tag(&pattern, &regex));
    }

    #[test]
    fn test_metadata_has_tag_inline() {
        let content = "# Title\n\ntags: #rust #cli";
        let metadata = Metadata {
            frontmatter: None,
            raw_content: content.to_string(),
        };

        let (pattern, regex) = ("rust".to_string(),
            regex::RegexBuilder::new(r"(^|[^[:word:]])#rust([^[:word:]]|$)")
                .case_insensitive(true)
                .build()
                .unwrap());

        assert!(metadata.has_tag(&pattern, &regex));
    }

    #[test]
    fn test_metadata_has_title_yaml() {
        let content = "---\ntitle: Meeting Notes\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);
        let metadata = Metadata {
            frontmatter: fm,
            raw_content: content.to_string(),
        };

        assert!(metadata.has_title("meeting"));
        assert!(!metadata.has_title("other"));
    }

    #[test]
    fn test_metadata_has_title_markdown() {
        let content = "# Meeting Notes 2025\n\nContent here";
        let metadata = Metadata {
            frontmatter: None,
            raw_content: content.to_string(),
        };

        assert!(metadata.has_title("meeting"));
        assert!(metadata.has_title("2025"));
        assert!(!metadata.has_title("other"));
    }

    #[test]
    fn test_metadata_has_field() {
        let content = "---\nauthor: John Doe\nstatus: draft\n---\n# Content";
        let path = PathBuf::from("test.md");
        let fm = extract_frontmatter(content, &path, false);
        let metadata = Metadata {
            frontmatter: fm,
            raw_content: content.to_string(),
        };

        assert!(metadata.has_field("author", "john"));
        assert!(metadata.has_field("status", "draft"));
        assert!(!metadata.has_field("author", "jane"));
    }

    #[test]
    fn test_matches_filename_with_regex() {
        let regex = regex::RegexBuilder::new("2025")
            .case_insensitive(false)
            .build()
            .unwrap();

        let path1 = PathBuf::from("notes-2025-01.md");
        let path2 = PathBuf::from("notes-2024.md");

        assert!(matches_filename(&path1, &regex));
        assert!(!matches_filename(&path2, &regex));
    }

    #[test]
    fn test_read_file_content_respects_head_lines() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        writeln!(temp_file, "Line 4").unwrap();
        writeln!(temp_file, "Line 5").unwrap();
        temp_file.flush().unwrap();

        let content = read_file_content(temp_file.path(), 3, false).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[2], "Line 3");
    }

    #[test]
    fn test_read_file_content_full_text() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        writeln!(temp_file, "Line 4").unwrap();
        writeln!(temp_file, "Line 5").unwrap();
        temp_file.flush().unwrap();

        let content = read_file_content(temp_file.path(), 3, true).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 5);
    }
}

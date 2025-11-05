use anyhow::{Context, Result};
use clap::Parser;
use globset::Glob;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
    #[arg(long = "glob", default_value = "*.md")]
    glob: String,

    /// Lines to scan for metadata
    #[arg(long = "head", default_value = "10")]
    head_lines: usize,

    /// Search full file content (not just first N lines)
    #[arg(long = "full-text")]
    full_text: bool,

    /// Directories to search (default: current directory)
    #[arg(default_value = ".")]
    dirs: Vec<PathBuf>,
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

struct Metadata {
    frontmatter: Option<Frontmatter>,
    raw_content: String,
    full_text: bool,
}

impl Metadata {
    fn from_file(path: &Path, head_lines: usize, full_text: bool) -> Result<Self> {
        // Read file content efficiently (only what we need)
        let content = read_file_content(path, head_lines, full_text)?;

        // Try to extract YAML frontmatter
        let frontmatter = extract_frontmatter(&content);

        // The content we read is already optimized for the mode
        Ok(Metadata {
            frontmatter,
            raw_content: content,
            full_text,
        })
    }

    fn has_tag(&self, pattern: &str) -> bool {
        // Remove leading # if present
        let pattern = pattern.strip_prefix('#').unwrap_or(pattern);

        // Check YAML frontmatter
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref tags) = fm.tags {
                if tags.contains_tag(pattern) {
                    return true;
                }
            }
        }

        let pattern_lower = pattern.to_lowercase();

        // In full-text mode, search for #tag anywhere in the content
        if self.full_text {
            let tag_with_hash = format!("#{}", pattern_lower);
            for line in self.raw_content.lines() {
                if line.to_lowercase().contains(&tag_with_hash) {
                    return true;
                }
            }
        } else {
            // In default mode, only check lines starting with "tags:"
            for line in self.raw_content.lines() {
                if line.to_lowercase().starts_with("tags:") {
                    if line.to_lowercase().contains(&pattern_lower) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn has_title(&self, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();

        // Check YAML frontmatter title
        if let Some(ref fm) = self.frontmatter {
            if let Some(ref title) = fm.title {
                if title.to_lowercase().contains(&pattern_lower) {
                    return true;
                }
            }
        }

        // Check markdown heading
        for line in self.raw_content.lines() {
            if line.starts_with("# ") {
                if line.to_lowercase().contains(&pattern_lower) {
                    return true;
                }
            }
        }

        false
    }

    fn has_field(&self, field_name: &str, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();

        // Check YAML frontmatter
        if let Some(ref fm) = self.frontmatter {
            if let Some(value) = fm.extra.get(field_name) {
                let value_str = match value {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::Sequence(seq) => {
                        return seq.iter().any(|v| {
                            if let serde_yaml::Value::String(s) = v {
                                s.to_lowercase().contains(&pattern_lower)
                            } else {
                                false
                            }
                        });
                    }
                    _ => return false,
                };

                if value_str.to_lowercase().contains(&pattern_lower) {
                    return true;
                }
            }
        }

        // Check simple inline format
        let field_prefix = format!("{}:", field_name).to_lowercase();
        for line in self.raw_content.lines() {
            if line.to_lowercase().starts_with(&field_prefix) {
                if line.to_lowercase().contains(&pattern_lower) {
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
        if line_count == 0 && line.trim() == "---" {
            in_frontmatter = true;
        } else if in_frontmatter && line.trim() == "---" {
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

fn extract_frontmatter(content: &str) -> Option<Frontmatter> {
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
    serde_yaml::from_str(&yaml_content).ok()
}

fn matches_filename(path: &Path, pattern: &str, ignore_case: bool) -> bool {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Try regex match first using RegexBuilder for proper case-insensitive support
    match RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
    {
        Ok(re) => re.is_match(filename),
        Err(_) => {
            // If pattern is not valid regex, fall back to substring matching
            if ignore_case {
                filename.to_lowercase().contains(&pattern.to_lowercase())
            } else {
                filename.contains(pattern)
            }
        }
    }
}

fn should_include_file(
    path: &Path,
    metadata: &Metadata,
    args: &Args,
) -> bool {
    // Check filename filters (fast, no file I/O)
    if !args.names.is_empty() {
        let name_matched = args.names.iter().any(|pattern| {
            matches_filename(path, pattern, args.ignore_case)
        });
        if !name_matched {
            return false;
        }
    }

    // Check tag filters
    if !args.tags.is_empty() {
        let tag_matched = args.tags.iter().any(|tag| metadata.has_tag(tag));
        if !tag_matched {
            return false;
        }
    }

    // Check title filters
    if !args.titles.is_empty() {
        let title_matched = args.titles.iter().any(|title| metadata.has_title(title));
        if !title_matched {
            return false;
        }
    }

    // Check field filters
    if !args.fields.is_empty() {
        let field_matched = args.fields.iter().any(|field_spec| {
            if let Some((field, pattern)) = field_spec.split_once(':') {
                metadata.has_field(field.trim(), pattern.trim())
            } else {
                false
            }
        });
        if !field_matched {
            return false;
        }
    }

    true
}

fn enumerate_files(args: &Args) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Build glob matcher from the glob pattern
    let glob_matcher = match Glob::new(&args.glob) {
        Ok(glob) => glob.compile_matcher(),
        Err(e) => {
            eprintln!("Warning: Invalid glob pattern '{}': {}", args.glob, e);
            // Fall back to matching *.md
            Glob::new("*.md").unwrap().compile_matcher()
        }
    };

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

        // Set max depth if specified
        if let Some(depth) = args.depth {
            walker.max_depth(Some(depth));
        }

        for entry in walker.build().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Additional filtering for specific directories we always want to skip
            // (in case they're not hidden or not in .gitignore)
            let path_str = path.to_string_lossy();
            if path_str.contains("/target/")
                || path_str.contains("/node_modules/")
                || path_str.contains("/.obsidian/") {
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

    files
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Enumerate all markdown files
    let mut files = enumerate_files(&args);

    // If no filters, just output all files
    if args.tags.is_empty()
        && args.titles.is_empty()
        && args.names.is_empty()
        && args.fields.is_empty()
    {
        // Sort results alphabetically (like ls)
        files.sort();

        for file in files {
            if args.nul {
                print!("{}\0", file.display());
            } else {
                println!("{}", file.display());
            }
        }
        return Ok(());
    }

    // Filter files in parallel
    let mut matching_files: Vec<PathBuf> = files
        .par_iter()
        .filter_map(|path| {
            // Extract metadata
            let metadata = Metadata::from_file(path, args.head_lines, args.full_text).ok()?;

            // Check filters
            if should_include_file(path, &metadata, &args) {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect();

    // Sort results alphabetically (like ls)
    matching_files.sort();

    // Output results
    for file in matching_files {
        if args.nul {
            print!("{}\0", file.display());
        } else {
            println!("{}", file.display());
        }
    }

    Ok(())
}

use crate::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test markdown file
fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(filename);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file_path
}

#[test]
fn test_output_files_with_multiple_files() {
    // Create a simple test to exercise output_files with real paths
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_file(&temp_dir, "file1.md", "# Test");
    let file2 = create_test_file(&temp_dir, "file2.md", "# Test");

    let files = vec![file1, file2];

    // Test both newline and NUL delimiters
    // These just ensure the function runs without panicking
    output_files(&files, false);
    output_files(&files, true);
}

#[test]
fn test_args_parsing_and_filter_creation() {
    // Test creating Args and CompiledFilters with various combinations
    let args = Args {
        tags: vec!["test".to_string()],
        titles: vec!["note".to_string()],
        authors: vec!["john".to_string()],
        names: vec!["file.*\\.md".to_string()],
        fields: vec!["status:draft".to_string()],
        nul: true,
        ignore_case: true,
        depth: Some(2),
        glob: "**/*.md".to_string(),
        head_lines: 20,
        full_text: true,
        verbose: true,
        date_after: Some("2025-01-01".to_string()),
        date_before: Some("2025-12-31".to_string()),
        dirs: vec![PathBuf::from(".")],
    };

    // Compile filters from args
    let filters = CompiledFilters::from_args(&args).unwrap();

    // Verify filters were created
    assert_eq!(filters.tag_patterns.len(), 1);
    assert_eq!(filters.title_patterns.len(), 1);
    assert_eq!(filters.author_patterns.len(), 1);
    assert_eq!(filters.name_patterns.len(), 1);
    assert_eq!(filters.field_patterns.len(), 1);
    assert!(filters.date_after.is_some());
    assert!(filters.date_before.is_some());
}

#[test]
fn test_metadata_extraction_from_file() {
    // Test Metadata::from_file which is called by find_matching_files
    let temp_dir = TempDir::new().unwrap();

    let file_path = create_test_file(
        &temp_dir,
        "test.md",
        "---\ntags: [rust]\nauthor: Test\n---\n# Content",
    );

    let metadata = Metadata::from_file(&file_path, 10, false, false).unwrap();

    // Verify metadata was extracted correctly
    let tag_regex = regex::Regex::new(r"(?i)\b#?rust\b").unwrap();
    assert!(metadata.has_tag("rust", &tag_regex));
    assert!(metadata.has_author("test"));
}

#[test]
fn test_compiled_filters_integration() {
    // Test CompiledFilters::from_args and filter matching
    let args = Args {
        tags: vec!["rust".to_string()],
        titles: vec!["test".to_string()],
        authors: vec![],
        names: vec![],
        fields: vec![],
        nul: false,
        ignore_case: false,
        depth: None,
        glob: "**/*.md".to_string(),
        head_lines: 10,
        full_text: false,
        verbose: false,
        date_after: None,
        date_before: None,
        dirs: vec![PathBuf::from(".")],
    };

    // Should compile filters successfully
    let filters = CompiledFilters::from_args(&args).unwrap();
    assert_eq!(filters.tag_patterns.len(), 1);
    assert_eq!(filters.title_patterns.len(), 1);
}

#[test]
fn test_read_file_content_with_metadata() {
    // Test read_file_content which is used throughout find_matching_files
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(
        &temp_dir,
        "test.md",
        "---\ntitle: Test\n---\n# Content\nLine 1\nLine 2",
    );

    // Test with head_lines limit
    let content = read_file_content(&file_path, 3, false).unwrap();
    assert!(content.contains("title"));

    // Test with full_text
    let content_full = read_file_content(&file_path, 3, true).unwrap();
    assert!(content_full.contains("Line 2"));
}

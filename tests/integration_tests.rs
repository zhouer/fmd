use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test markdown file with content
fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// Helper to run fmd command and get output
fn run_fmd(args: &[&str], dir: &TempDir) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_fmd"))
        .args(args)
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute fmd");

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_list_all_markdown_files() {
    let temp_dir = TempDir::new().unwrap();
    create_test_file(&temp_dir, "file1.md", "# Hello");
    create_test_file(&temp_dir, "file2.md", "# World");
    create_test_file(&temp_dir, "file3.txt", "Not markdown");

    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("file1.md"));
    assert!(output.contains("file2.md"));
    assert!(!output.contains("file3.txt"));
}

#[test]
fn test_filter_by_yaml_tag() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "with_tag.md",
        "---\ntags: [rust, cli]\n---\n# Content");
    create_test_file(&temp_dir, "without_tag.md",
        "---\ntags: [python]\n---\n# Content");

    let output = run_fmd(&["--tag", "rust"], &temp_dir);

    assert!(output.contains("with_tag.md"));
    assert!(!output.contains("without_tag.md"));
}

#[test]
fn test_filter_by_inline_tag() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "with_inline.md",
        "# My Note\n\ntags: #rust #programming");
    create_test_file(&temp_dir, "without_inline.md",
        "# Other Note\n\ntags: #python");

    let output = run_fmd(&["--tag", "rust"], &temp_dir);

    assert!(output.contains("with_inline.md"));
    assert!(!output.contains("without_inline.md"));
}

#[test]
fn test_filter_by_yaml_title() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "meeting.md",
        "---\ntitle: Meeting Notes\n---\n# Content");
    create_test_file(&temp_dir, "other.md",
        "---\ntitle: Other Document\n---\n# Content");

    let output = run_fmd(&["--title", "meeting"], &temp_dir);

    assert!(output.contains("meeting.md"));
    assert!(!output.contains("other.md"));
}

#[test]
fn test_filter_by_markdown_heading() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "meeting.md", "# Meeting Notes 2025");
    create_test_file(&temp_dir, "other.md", "# Other Document");

    let output = run_fmd(&["--title", "meeting"], &temp_dir);

    assert!(output.contains("meeting.md"));
    assert!(!output.contains("other.md"));
}

#[test]
fn test_filter_by_filename() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "2025-01-notes.md", "# Content");
    create_test_file(&temp_dir, "2024-12-notes.md", "# Content");

    let output = run_fmd(&["--name", "2025"], &temp_dir);

    assert!(output.contains("2025-01-notes.md"));
    assert!(!output.contains("2024-12-notes.md"));
}

#[test]
fn test_filter_by_custom_field() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "johns_note.md",
        "---\nauthor: John Doe\n---\n# Content");
    create_test_file(&temp_dir, "janes_note.md",
        "---\nauthor: Jane Smith\n---\n# Content");

    let output = run_fmd(&["--field", "author:John"], &temp_dir);

    assert!(output.contains("johns_note.md"));
    assert!(!output.contains("janes_note.md"));
}

#[test]
fn test_combined_filters_and_logic() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "match_both.md",
        "---\ntags: [rust]\nauthor: John\n---\n# Content");
    create_test_file(&temp_dir, "match_tag_only.md",
        "---\ntags: [rust]\nauthor: Jane\n---\n# Content");
    create_test_file(&temp_dir, "match_author_only.md",
        "---\ntags: [python]\nauthor: John\n---\n# Content");

    let output = run_fmd(&["--tag", "rust", "--field", "author:John"], &temp_dir);

    assert!(output.contains("match_both.md"));
    assert!(!output.contains("match_tag_only.md"));
    assert!(!output.contains("match_author_only.md"));
}

#[test]
fn test_multiple_tags_or_logic() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "rust.md",
        "---\ntags: [rust]\n---\n# Content");
    create_test_file(&temp_dir, "python.md",
        "---\ntags: [python]\n---\n# Content");
    create_test_file(&temp_dir, "java.md",
        "---\ntags: [java]\n---\n# Content");

    let output = run_fmd(&["--tag", "rust", "--tag", "python"], &temp_dir);

    assert!(output.contains("rust.md"));
    assert!(output.contains("python.md"));
    assert!(!output.contains("java.md"));
}

#[test]
fn test_case_insensitive_filename() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "README.md", "# Content");
    create_test_file(&temp_dir, "notes.md", "# Content");

    let output = run_fmd(&["-i", "--name", "readme"], &temp_dir);

    assert!(output.contains("README.md"));
    assert!(!output.contains("notes.md"));
}

#[test]
fn test_nul_delimited_output() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "file1.md", "# Content");
    create_test_file(&temp_dir, "file2.md", "# Content");

    let output = Command::new(env!("CARGO_BIN_EXE_fmd"))
        .arg("-0")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute fmd");

    let output_bytes = output.stdout;
    assert!(output_bytes.contains(&0)); // Contains NUL bytes
}

#[test]
fn test_empty_frontmatter() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "empty_fm.md", "---\n---\n# Content");

    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("empty_fm.md"));
}

#[test]
fn test_malformed_frontmatter() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "malformed.md", "---\ninvalid: [yaml\n---\n# Content");

    // Should not crash, should handle gracefully
    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("malformed.md"));
}

#[test]
fn test_multiline_yaml_tags() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir, "multiline.md",
        "---\ntags:\n  - rust\n  - cli\n  - tools\n---\n# Content");

    let output = run_fmd(&["--tag", "cli"], &temp_dir);

    assert!(output.contains("multiline.md"));
}

#[test]
fn test_full_text_search() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with tag deep in content (beyond default 10 lines)
    let mut deep_content = String::new();
    deep_content.push_str("# Title\n\n");
    for i in 1..=15 {
        deep_content.push_str(&format!("Line {} of content.\n", i));
    }
    deep_content.push_str("And here we mention #important");

    create_test_file(&temp_dir, "deep_tag.md", &deep_content);

    // Without --full-text, should not find tag deep in content (beyond line 10)
    let output_default = run_fmd(&["--tag", "important"], &temp_dir);
    assert!(!output_default.contains("deep_tag.md"));

    // With --full-text, should find it
    let output_fulltext = run_fmd(&["--tag", "important", "--full-text"], &temp_dir);
    assert!(output_fulltext.contains("deep_tag.md"));
}

#[test]
fn test_skip_target_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create target directory
    fs::create_dir(temp_dir.path().join("target")).unwrap();
    create_test_file(&temp_dir, "root.md", "# Root");

    let target_dir = temp_dir.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();
    let mut file = fs::File::create(target_dir.join("inside.md")).unwrap();
    file.write_all(b"# Inside Target").unwrap();

    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("root.md"));
    assert!(!output.contains("inside.md"));
}

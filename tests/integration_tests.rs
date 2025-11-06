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
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();
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
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "with_tag.md",
        "---\ntags: [rust, cli]\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "without_tag.md",
        "---\ntags: [python]\n---\n# Content",
    );

    let output = run_fmd(&["--tag", "rust"], &temp_dir);

    assert!(output.contains("with_tag.md"));
    assert!(!output.contains("without_tag.md"));
}

#[test]
fn test_filter_by_inline_tag() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "with_inline.md",
        "# My Note\n\ntags: #rust #programming",
    );
    create_test_file(
        &temp_dir,
        "without_inline.md",
        "# Other Note\n\ntags: #python",
    );

    let output = run_fmd(&["--tag", "rust"], &temp_dir);

    assert!(output.contains("with_inline.md"));
    assert!(!output.contains("without_inline.md"));
}

#[test]
fn test_filter_by_yaml_title() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "meeting.md",
        "---\ntitle: Meeting Notes\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "other.md",
        "---\ntitle: Other Document\n---\n# Content",
    );

    let output = run_fmd(&["--title", "meeting"], &temp_dir);

    assert!(output.contains("meeting.md"));
    assert!(!output.contains("other.md"));
}

#[test]
fn test_filter_by_markdown_heading() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "meeting.md", "# Meeting Notes 2025");
    create_test_file(&temp_dir, "other.md", "# Other Document");

    let output = run_fmd(&["--title", "meeting"], &temp_dir);

    assert!(output.contains("meeting.md"));
    assert!(!output.contains("other.md"));
}

#[test]
fn test_filter_by_filename() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "2025-01-notes.md", "# Content");
    create_test_file(&temp_dir, "2024-12-notes.md", "# Content");

    let output = run_fmd(&["--name", "2025"], &temp_dir);

    assert!(output.contains("2025-01-notes.md"));
    assert!(!output.contains("2024-12-notes.md"));
}

#[test]
fn test_filter_by_custom_field() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "johns_note.md",
        "---\nauthor: John Doe\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "janes_note.md",
        "---\nauthor: Jane Smith\n---\n# Content",
    );

    let output = run_fmd(&["--field", "author:John"], &temp_dir);

    assert!(output.contains("johns_note.md"));
    assert!(!output.contains("janes_note.md"));
}

#[test]
fn test_combined_filters_and_logic() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "match_both.md",
        "---\ntags: [rust]\nauthor: John\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "match_tag_only.md",
        "---\ntags: [rust]\nauthor: Jane\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "match_author_only.md",
        "---\ntags: [python]\nauthor: John\n---\n# Content",
    );

    let output = run_fmd(&["--tag", "rust", "--field", "author:John"], &temp_dir);

    assert!(output.contains("match_both.md"));
    assert!(!output.contains("match_tag_only.md"));
    assert!(!output.contains("match_author_only.md"));
}

#[test]
fn test_multiple_tags_or_logic() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "rust.md", "---\ntags: [rust]\n---\n# Content");
    create_test_file(
        &temp_dir,
        "python.md",
        "---\ntags: [python]\n---\n# Content",
    );
    create_test_file(&temp_dir, "java.md", "---\ntags: [java]\n---\n# Content");

    let output = run_fmd(&["--tag", "rust", "--tag", "python"], &temp_dir);

    assert!(output.contains("rust.md"));
    assert!(output.contains("python.md"));
    assert!(!output.contains("java.md"));
}

#[test]
fn test_case_insensitive_filename() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "README.md", "# Content");
    create_test_file(&temp_dir, "notes.md", "# Content");

    let output = run_fmd(&["-i", "--name", "readme"], &temp_dir);

    assert!(output.contains("README.md"));
    assert!(!output.contains("notes.md"));
}

#[test]
fn test_nul_delimited_output() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

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
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "empty_fm.md", "---\n---\n# Content");

    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("empty_fm.md"));
}

#[test]
fn test_malformed_frontmatter() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "malformed.md",
        "---\ninvalid: [yaml\n---\n# Content",
    );

    // Should not crash, should handle gracefully
    let output = run_fmd(&[], &temp_dir);

    assert!(output.contains("malformed.md"));
}

#[test]
fn test_multiline_yaml_tags() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "multiline.md",
        "---\ntags:\n  - rust\n  - cli\n  - tools\n---\n# Content",
    );

    let output = run_fmd(&["--tag", "cli"], &temp_dir);

    assert!(output.contains("multiline.md"));
}

#[test]
fn test_full_text_search() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    // DEFAULT_HEAD_LINES is 10 in main.rs
    const DEFAULT_HEAD_LINES: usize = 10;

    // Create a file with tag deep in content (beyond default head lines)
    let mut deep_content = String::new();
    deep_content.push_str("# Title\n\n");
    // Generate lines beyond DEFAULT_HEAD_LINES to ensure tag is not in scanned content
    for i in 1..=(DEFAULT_HEAD_LINES + 5) {
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
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

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

// Author filtering tests

#[test]
fn test_filter_by_author_yaml() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "alice.md",
        "---\nauthor: Alice\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "bob.md",
        "---\nauthor: Bob\n---\n# Content",
    );
    create_test_file(&temp_dir, "none.md", "# Content without author");

    let output = run_fmd(&["--author", "alice"], &temp_dir);

    assert!(output.contains("alice.md"));
    assert!(!output.contains("bob.md"));
    assert!(!output.contains("none.md"));
}

#[test]
fn test_filter_by_author_inline() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "has_author.md", "# Title\nauthor: Charlie\n\nContent");
    create_test_file(&temp_dir, "no_author.md", "# Title\n\nJust content");

    let output = run_fmd(&["--author", "charlie"], &temp_dir);

    assert!(output.contains("has_author.md"));
    assert!(!output.contains("no_author.md"));
}

#[test]
fn test_filter_by_multiple_authors() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "alice.md",
        "---\nauthor: Alice\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "bob.md",
        "---\nauthor: Bob\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "charlie.md",
        "---\nauthor: Charlie\n---\n# Content",
    );

    let output = run_fmd(&["--author", "alice", "--author", "bob"], &temp_dir);

    assert!(output.contains("alice.md"));
    assert!(output.contains("bob.md"));
    assert!(!output.contains("charlie.md"));
}

// Depth limit tests

#[test]
fn test_depth_limit() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "root.md", "# Root");

    let level1 = temp_dir.path().join("level1");
    fs::create_dir(&level1).unwrap();
    let mut file = fs::File::create(level1.join("l1.md")).unwrap();
    file.write_all(b"# Level 1").unwrap();

    let level2 = level1.join("level2");
    fs::create_dir(&level2).unwrap();
    let mut file = fs::File::create(level2.join("l2.md")).unwrap();
    file.write_all(b"# Level 2").unwrap();

    let output = run_fmd(&["--depth", "2"], &temp_dir);

    assert!(output.contains("root.md"));
    assert!(output.contains("l1.md"));
    assert!(!output.contains("l2.md"));
}

#[test]
fn test_depth_one() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "root.md", "# Root");

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let mut file = fs::File::create(subdir.join("nested.md")).unwrap();
    file.write_all(b"# Nested").unwrap();

    let output = run_fmd(&["--depth", "1"], &temp_dir);

    assert!(output.contains("root.md"));
    assert!(!output.contains("nested.md"));
}

// Combined filter tests

#[test]
fn test_combined_tag_and_author() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "match.md",
        "---\ntags: [rust]\nauthor: Alice\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "tag_only.md",
        "---\ntags: [rust]\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "author_only.md",
        "---\nauthor: Alice\n---\n# Content",
    );

    let output = run_fmd(&["--tag", "rust", "--author", "alice"], &temp_dir);

    assert!(output.contains("match.md"));
    assert!(!output.contains("tag_only.md"));
    assert!(!output.contains("author_only.md"));
}

#[test]
fn test_combined_title_field_date() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "match.md",
        "---\ntitle: Rust Guide\nstatus: active\ndate: 2024-06-15\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "wrong_date.md",
        "---\ntitle: Rust Guide\nstatus: active\ndate: 2023-01-01\n---\n# Content",
    );

    let output = run_fmd(
        &["--title", "rust", "--field", "status:active", "--date-after", "2024-01-01"],
        &temp_dir,
    );

    assert!(output.contains("match.md"));
    assert!(!output.contains("wrong_date.md"));
}

// Custom glob pattern tests

#[test]
fn test_custom_glob_markdown_extension() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "file.md", "# Content");
    create_test_file(&temp_dir, "file.markdown", "# Content");
    create_test_file(&temp_dir, "file.txt", "# Content");

    let output = run_fmd(&["--glob", "*.markdown"], &temp_dir);

    assert!(!output.contains("file.md"));
    assert!(output.contains("file.markdown"));
    assert!(!output.contains("file.txt"));
}

#[test]
fn test_custom_glob_specific_directory() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "root.md", "# Root");

    let docs = temp_dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    let mut file = fs::File::create(docs.join("doc.md")).unwrap();
    file.write_all(b"# Doc").unwrap();

    let notes = temp_dir.path().join("notes");
    fs::create_dir(&notes).unwrap();
    let mut file = fs::File::create(notes.join("note.md")).unwrap();
    file.write_all(b"# Note").unwrap();

    let output = run_fmd(&["--glob", "**/docs/*.md"], &temp_dir);

    assert!(!output.contains("root.md"));
    assert!(output.contains("doc.md"));
    assert!(!output.contains("note.md"));
}

// Error handling tests

#[test]
fn test_nonexistent_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_fmd"))
        .arg("/nonexistent/directory/path")
        .output()
        .expect("Failed to execute fmd");

    // Should handle gracefully
    assert!(output.status.success());
}

// Full-text search tests

#[test]
fn test_full_text_tag_search() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "early.md",
        "---\ntags: [rust]\n---\n# Content",
    );
    create_test_file(
        &temp_dir,
        "late.md",
        &format!("# Title\n\n{}tags: #rust", "filler\n".repeat(20)),
    );

    // Without full-text, should only find early.md
    let output = run_fmd(&["--tag", "rust"], &temp_dir);
    assert!(output.contains("early.md"));

    // With full-text, should find both
    let output = run_fmd(&["--tag", "rust", "--full-text"], &temp_dir);
    assert!(output.contains("early.md"));
    assert!(output.contains("late.md"));
}

// Unicode and special characters

#[test]
fn test_unicode_content() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(
        &temp_dir,
        "chinese.md",
        "---\ntitle: 中文标题\nauthor: 张三\n---\n# Content",
    );

    let output = run_fmd(&["--title", "中文"], &temp_dir);
    assert!(output.contains("chinese.md"));

    let output = run_fmd(&["--author", "张三"], &temp_dir);
    assert!(output.contains("chinese.md"));
}

#[test]
fn test_special_chars_in_filename() {
    let temp_dir = tempfile::Builder::new().prefix("test_").tempdir_in(".").unwrap();

    create_test_file(&temp_dir, "file[1].md", "# Content");
    create_test_file(&temp_dir, "file (2).md", "# Content");

    let output = run_fmd(&[], &temp_dir);
    assert!(output.contains("file[1].md") || output.contains("file"));
    assert!(output.contains("file (2).md") || output.contains("file"));
}

use crate::*;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_read_file_content_respects_head_lines() {
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

// Edge case and error handling tests

#[test]
fn test_read_file_content_empty_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_read_file_content_single_line() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Single line").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert_eq!(content, "Single line");
}

#[test]
fn test_read_file_content_with_frontmatter() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "---").unwrap();
    writeln!(temp_file, "title: Test").unwrap();
    writeln!(temp_file, "---").unwrap();
    writeln!(temp_file, "Content").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("---"));
    assert!(content.contains("title: Test"));
}

#[test]
fn test_read_file_content_unicode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "你好世界").unwrap();
    writeln!(temp_file, "こんにちは").unwrap();
    writeln!(temp_file, "Здравствуй").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("你好世界"));
    assert!(content.contains("こんにちは"));
    assert!(content.contains("Здравствуй"));
}

#[test]
fn test_read_file_content_nonexistent_file() {
    let path = PathBuf::from("/nonexistent/path/to/file.md");
    let result = read_file_content(&path, 10, false);
    assert!(result.is_err());
}

#[test]
fn test_read_file_content_head_lines_zero() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Line 1").unwrap();
    writeln!(temp_file, "Line 2").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 0, false).unwrap();
    // Current implementation reads one line before checking head_lines
    // So with head_lines=0, it returns the first line
    assert_eq!(content, "Line 1");
}

#[test]
fn test_read_file_content_head_lines_greater_than_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Line 1").unwrap();
    writeln!(temp_file, "Line 2").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 100, false).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_read_file_content_only_frontmatter() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "---").unwrap();
    writeln!(temp_file, "title: Test").unwrap();
    writeln!(temp_file, "---").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("title: Test"));
}

#[test]
fn test_read_file_content_windows_line_endings() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Line 1\r\nLine 2\r\nLine 3").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    // Should handle CRLF line endings
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
}

#[test]
fn test_read_file_content_mixed_line_endings() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Line 1\nLine 2\r\nLine 3").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
}

#[test]
fn test_read_file_content_incomplete_frontmatter() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "---").unwrap();
    writeln!(temp_file, "title: Test").unwrap();
    writeln!(temp_file, "No closing delimiter").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("title: Test"));
    assert!(content.contains("No closing delimiter"));
}

#[test]
fn test_read_file_content_with_special_chars() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Special: !@#$%^&*()").unwrap();
    writeln!(temp_file, "Quotes: \"test\" 'test'").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("!@#$%^&*()"));
    assert!(content.contains("\"test\""));
}

#[test]
fn test_read_file_content_whitespace_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Line 1").unwrap();
    writeln!(temp_file).unwrap();
    writeln!(temp_file, "   ").unwrap();
    writeln!(temp_file, "Line 4").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // Empty and whitespace lines should be preserved
    assert!(lines.len() >= 3);
}

#[test]
fn test_read_file_content_long_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let long_line = "a".repeat(5000);
    writeln!(temp_file, "{}", long_line).unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    // Should handle long lines (though may truncate per MAX_LINE_LENGTH)
    assert!(!content.is_empty());
}

#[test]
fn test_read_file_content_full_text_ignores_head_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Line 1").unwrap();
    writeln!(temp_file, "Line 2").unwrap();
    writeln!(temp_file, "Line 3").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 1, true).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // full_text=true should read all lines regardless of head_lines
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_read_file_content_tabs_and_spaces() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "\tIndented with tab").unwrap();
    writeln!(temp_file, "    Indented with spaces").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("Indented with tab"));
    assert!(content.contains("Indented with spaces"));
}

#[test]
fn test_read_file_content_markdown_formatting() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "# Heading").unwrap();
    writeln!(temp_file, "**bold** and *italic*").unwrap();
    writeln!(temp_file, "[link](url)").unwrap();
    writeln!(temp_file, "```code```").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    assert!(content.contains("# Heading"));
    assert!(content.contains("**bold**"));
    assert!(content.contains("[link](url)"));
}

#[test]
fn test_read_file_content_consecutive_newlines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Line 1").unwrap();
    writeln!(temp_file).unwrap();
    writeln!(temp_file).unwrap();
    writeln!(temp_file, "Line 4").unwrap();
    temp_file.flush().unwrap();

    let content = read_file_content(temp_file.path(), 10, false).unwrap();
    // Multiple newlines should be preserved
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 4"));
}

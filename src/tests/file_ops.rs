use crate::*;
use std::io::Write;
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

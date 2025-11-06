use crate::*;
use std::path::PathBuf;

#[test]
fn test_extract_frontmatter_valid() {
    let content = "---\ntitle: Test\ntags: [rust, cli]\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_empty() {
    let content = "---\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_none());
}

#[test]
fn test_extract_frontmatter_none() {
    let content = "# Just a heading";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_none());
}

#[test]
fn test_extract_frontmatter_multiline_tags() {
    let content = "---\ntags:\n  - rust\n  - cli\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

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

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

// Edge case tests

#[test]
fn test_extract_frontmatter_no_closing_delimiter() {
    let content = "---\ntitle: Test\ntags: [rust]";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Without closing delimiter, it parses all remaining lines as YAML
    // If valid YAML, returns Some; if invalid, returns None
    // This content is valid YAML, so should return Some
    assert!(fm.is_some());
    assert_eq!(fm.unwrap().title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_whitespace_around_delimiters() {
    let content = "  ---  \ntitle: Test\n  ---  \n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_with_unicode() {
    let content = "---\ntitle: 测试文档\nauthor: 张三\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("测试文档".to_string()));
    assert_eq!(fm.author, Some("张三".to_string()));
}

#[test]
fn test_extract_frontmatter_with_special_yaml_types() {
    let content = "---\ntitle: Test\ncount: 42\nenabled: true\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_nested_yaml() {
    let content = "---\ntitle: Test\nmetadata:\n  author: John\n  date: 2024\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Should still parse, nested fields accessible via other_fields
    assert!(fm.is_some());
}

#[test]
fn test_extract_frontmatter_malformed_yaml() {
    let content = "---\ntitle: Test\ninvalid: [unclosed\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Should return None for malformed YAML
    assert!(fm.is_none());
}

#[test]
fn test_extract_frontmatter_quoted_strings() {
    let content = "---\ntitle: \"Test: With Colon\"\nauthor: 'Single Quotes'\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test: With Colon".to_string()));
    assert_eq!(fm.author, Some("Single Quotes".to_string()));
}

#[test]
fn test_extract_frontmatter_multiline_string() {
    let content = "---\ntitle: |\n  Multi\n  Line\n  Title\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    // YAML multiline strings should be parsed
    assert!(fm.title.is_some());
}

#[test]
fn test_extract_frontmatter_empty_values() {
    let content = "---\ntitle:\nauthor:\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Empty YAML values become None
    assert!(fm.is_some());
}

#[test]
fn test_extract_frontmatter_tags_with_special_chars() {
    let content = "---\ntags: [rust-2024, 'C++', '#hashtag']\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    if let Some(TagValue::Array(tags)) = fm.tags {
        assert!(tags.contains(&"rust-2024".to_string()));
        assert!(tags.contains(&"C++".to_string()));
        assert!(tags.contains(&"#hashtag".to_string()));
    }
}

#[test]
fn test_extract_frontmatter_date_field() {
    let content = "---\ndate: 2024-01-15\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    // Date should be accessible via other_fields
}

#[test]
fn test_extract_frontmatter_multiple_date_fields() {
    let content = "---\ndate: 2024-01-15\ncreated: 2024-01-01\nmodified: 2024-01-20\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
}

#[test]
fn test_extract_frontmatter_content_after() {
    let content = "---\ntitle: Test\n---\n# Heading\n\nSome content\n\n---\nNot frontmatter";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_no_content_after() {
    let content = "---\ntitle: Test\n---";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
}

#[test]
fn test_extract_frontmatter_leading_whitespace_in_content() {
    let content = "---\n  title: Test\n  tags: [rust]\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
}

#[test]
fn test_extract_frontmatter_tabs_in_yaml() {
    let content = "---\ntitle:\tTest With Tabs\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // YAML doesn't officially support tabs, but may parse
    assert!(fm.is_some());
}

#[test]
fn test_extract_frontmatter_starts_with_whitespace() {
    let content = "\n\n---\ntitle: Test\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Should return None if frontmatter doesn't start on first line
    assert!(fm.is_none());
}

#[test]
fn test_extract_frontmatter_only_opening_delimiter() {
    let content = "---\n# Just content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);

    // Without closing delimiter, it parses "# Just content" as YAML
    // In YAML, lines starting with # are comments, so empty content is valid
    // Empty YAML content parses successfully, returning Some with default values
    assert!(fm.is_some());
}

#[test]
fn test_tag_value_single_empty() {
    let single = TagValue::Single("".to_string());
    assert!(single.contains_tag(""));
    assert!(!single.contains_tag("anything"));
}

#[test]
fn test_tag_value_array_empty() {
    let array = TagValue::Array(vec![]);
    assert!(!array.contains_tag("anything"));
}

#[test]
fn test_tag_value_array_with_empty_strings() {
    let array = TagValue::Array(vec!["".to_string(), "rust".to_string()]);
    assert!(array.contains_tag("rust"));
    assert!(array.contains_tag(""));
}

#[test]
fn test_tag_value_partial_match() {
    let single = TagValue::Single("rust-programming".to_string());
    assert!(single.contains_tag("rust"));
    assert!(single.contains_tag("programming"));
    assert!(single.contains_tag("rust-prog"));
}

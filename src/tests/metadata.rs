use crate::*;
use std::path::PathBuf;

#[test]
fn test_metadata_has_tag_yaml() {
    let content = "---\ntags: [rust, cli]\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    let (pattern, regex) = (
        "rust".to_string(),
        regex::RegexBuilder::new(r"(^|[^[:word:]])#rust([^[:word:]]|$)")
            .case_insensitive(true)
            .build()
            .unwrap(),
    );

    assert!(metadata.has_tag(&pattern, &regex));
}

#[test]
fn test_metadata_has_tag_inline() {
    let content = "# Title\n\ntags: #rust #cli";
    let metadata = Metadata {
        frontmatter: None,
        raw_content: content.to_string(),
    };

    let (pattern, regex) = (
        "rust".to_string(),
        regex::RegexBuilder::new(r"(^|[^[:word:]])#rust([^[:word:]]|$)")
            .case_insensitive(true)
            .build()
            .unwrap(),
    );

    assert!(metadata.has_tag(&pattern, &regex));
}

#[test]
fn test_metadata_has_title_yaml() {
    let content = "---\ntitle: Meeting Notes\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
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
    let fm = extract_frontmatter(content, &path);
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

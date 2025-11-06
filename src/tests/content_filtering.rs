use crate::{extract_frontmatter, should_include_file_by_content, CompiledFilters, Metadata};
use chrono::NaiveDate;
use regex::Regex;
use std::path::PathBuf;

fn create_test_metadata(content: &str) -> Metadata {
    let path = PathBuf::from("test.md");
    Metadata {
        frontmatter: extract_frontmatter(content, &path),
        raw_content: content.to_string(),
    }
}

#[test]
fn filter_types_basic_match_and_no_match() {
    // Table-driven test for all filter types (match and no-match pairs)
    let test_cases = vec![
        // Tag filters
        (
            r#"---
tags: [rust, programming]
---"#,
            CompiledFilters {
                tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            true,
        ),
        (
            r#"---
tags: [python, java]
---"#,
            CompiledFilters {
                tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            false,
        ),
        // Title filters
        (
            r#"---
title: My Rust Guide
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec!["rust".to_string()],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            true,
        ),
        (
            r#"---
title: Python Tutorial
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec!["rust".to_string()],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            false,
        ),
        // Author filters
        (
            r#"---
author: John Doe
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec!["john".to_string()],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            true,
        ),
        (
            r#"---
author: Jane Smith
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec!["john".to_string()],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: None,
            },
            false,
        ),
        // Field filters
        (
            r#"---
status: active
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![("status".to_string(), "active".to_string())],
                date_after: None,
                date_before: None,
            },
            true,
        ),
        (
            r#"---
status: draft
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![("status".to_string(), "active".to_string())],
                date_after: None,
                date_before: None,
            },
            false,
        ),
    ];

    for (content, filters, expected) in test_cases {
        let metadata = create_test_metadata(content);
        assert_eq!(
            should_include_file_by_content(&metadata, &filters),
            expected,
            "Failed for content: {}",
            content
        );
    }
}

#[test]
fn filter_multiple_patterns_or_logic() {
    // Test OR logic for multiple patterns of the same type
    let content = r#"---
tags: [rust]
---"#;
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![
            ("rust".to_string(), Regex::new("rust").unwrap()),
            ("python".to_string(), Regex::new("python").unwrap()),
        ],
        title_patterns: vec![],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: None,
        date_before: None,
    };

    // OR logic: should match if ANY tag matches
    assert!(should_include_file_by_content(&metadata, &filters));
}

#[test]
fn date_filtering_after_and_before() {
    let test_cases = vec![
        // Date after - match
        (
            r#"---
date: 2024-01-15
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                date_before: None,
            },
            true,
        ),
        // Date after - no match
        (
            r#"---
date: 2023-12-15
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                date_before: None,
            },
            false,
        ),
        // Date before - match
        (
            r#"---
date: 2023-12-15
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            },
            true,
        ),
        // Date before - no match
        (
            r#"---
date: 2024-02-15
---"#,
            CompiledFilters {
                tag_patterns: vec![],
                title_patterns: vec![],
                author_patterns: vec![],
                name_patterns: vec![],
                field_patterns: vec![],
                date_after: None,
                date_before: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            },
            false,
        ),
    ];

    for (content, filters, expected) in test_cases {
        let metadata = create_test_metadata(content);
        assert_eq!(
            should_include_file_by_content(&metadata, &filters),
            expected,
            "Failed for content: {}",
            content
        );
    }
}

#[test]
fn date_filtering_range() {
    let test_cases = vec![
        // In range
        (
            r#"---
date: 2024-01-15
---"#,
            true,
        ),
        // Out of range
        (
            r#"---
date: 2024-03-15
---"#,
            false,
        ),
    ];

    for (content, expected) in test_cases {
        let metadata = create_test_metadata(content);
        let filters = CompiledFilters {
            tag_patterns: vec![],
            title_patterns: vec![],
            author_patterns: vec![],
            name_patterns: vec![],
            field_patterns: vec![],
            date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            date_before: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
        };

        assert_eq!(
            should_include_file_by_content(&metadata, &filters),
            expected
        );
    }
}

#[test]
fn combined_filters_all_match() {
    let content = r#"---
title: Rust Guide
tags: [rust, programming]
author: John Doe
status: active
date: 2024-01-15
---"#;
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
        title_patterns: vec!["rust".to_string()],
        author_patterns: vec!["john".to_string()],
        name_patterns: vec![],
        field_patterns: vec![("status".to_string(), "active".to_string())],
        date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        date_before: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
    };

    // AND logic: all filters must match
    assert!(should_include_file_by_content(&metadata, &filters));
}

#[test]
fn combined_filters_one_fails() {
    let content = r#"---
title: Rust Guide
tags: [rust, programming]
author: John Doe
status: draft
date: 2024-01-15
---"#;
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
        title_patterns: vec!["rust".to_string()],
        author_patterns: vec!["john".to_string()],
        name_patterns: vec![],
        field_patterns: vec![("status".to_string(), "active".to_string())],
        date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        date_before: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
    };

    // Should fail because status doesn't match (draft != active)
    assert!(!should_include_file_by_content(&metadata, &filters));
}

#[test]
fn date_filter_no_date_in_content() {
    let content = r#"---
title: Test
---"#;
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![],
        title_patterns: vec![],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        date_before: None,
    };

    // Should not match if date filter is specified but no date in content
    assert!(!should_include_file_by_content(&metadata, &filters));
}

#[test]
fn empty_frontmatter_no_filters() {
    let content = "Just plain content";
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![],
        title_patterns: vec![],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: None,
        date_before: None,
    };

    // No filters should include everything
    assert!(should_include_file_by_content(&metadata, &filters));
}

#[test]
fn empty_frontmatter_with_tag_filter() {
    let content = "Just plain content";
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
        title_patterns: vec![],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: None,
        date_before: None,
    };

    // Should not match without tags
    assert!(!should_include_file_by_content(&metadata, &filters));
}

#[test]
fn inline_tag_match() {
    let content = "This is about #rust programming";
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![("rust".to_string(), Regex::new("rust").unwrap())],
        title_patterns: vec![],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: None,
        date_before: None,
    };

    assert!(should_include_file_by_content(&metadata, &filters));
}

#[test]
fn markdown_heading_title_match() {
    let content = "# Rust Programming Guide\n\nContent here";
    let metadata = create_test_metadata(content);
    let filters = CompiledFilters {
        tag_patterns: vec![],
        title_patterns: vec!["rust".to_string()],
        author_patterns: vec![],
        name_patterns: vec![],
        field_patterns: vec![],
        date_after: None,
        date_before: None,
    };

    assert!(should_include_file_by_content(&metadata, &filters));
}

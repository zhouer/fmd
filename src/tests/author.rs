use crate::{extract_frontmatter, Metadata};
use std::path::PathBuf;

#[test]
fn has_author_yaml_comprehensive() {
    // Test exact match, partial match, case insensitivity, no match, and missing author
    let test_cases = vec![
        (
            r#"---
author: John Doe
---
Some content here."#,
            vec![("john doe", true), ("john", true), ("doe", true), ("bob", false)],
        ),
        (
            r#"---
author: Jane Smith
---
Content"#,
            vec![
                ("jane", true),
                ("jane", true),  // case insensitive
                ("jane smith", true),  // case insensitive
                ("smith", true),
                ("xyz", false),
            ],
        ),
        (
            r#"---
title: Test Document
tags: [rust, testing]
---
Content"#,
            vec![("anyone", false)],
        ),
    ];

    for (content, expectations) in test_cases {
        let path = PathBuf::from("test.md");
        let metadata = Metadata {
            frontmatter: extract_frontmatter(content, &path),
            raw_content: content.to_string(),
        };

        for (pattern, should_match) in expectations {
            assert_eq!(
                metadata.has_author(pattern),
                should_match,
                "Pattern '{}' in content '{}'",
                pattern,
                content
            );
        }
    }
}

#[test]
fn has_author_inline_comprehensive() {
    // Test inline author detection with case insensitivity and whitespace handling
    let test_cases = vec![
        (
            r#"Some content
author: Bob Johnson
More content here"#,
            vec![("bob", true), ("johnson", true), ("bob johnson", true)],
        ),
        (
            r#"First line
Author: Carol Williams
Last line"#,
            vec![
                ("carol", true),
                ("carol", true),  // case insensitive
                ("williams", true),
                ("williams", true),  // case insensitive
            ],
        ),
        (
            r#"Content
    author: David Brown
More content"#,
            vec![("david", true), ("brown", true)],
        ),
    ];

    for (content, expectations) in test_cases {
        let path = PathBuf::from("test.md");
        let metadata = Metadata {
            frontmatter: extract_frontmatter(content, &path),
            raw_content: content.to_string(),
        };

        for (pattern, should_match) in expectations {
            assert_eq!(
                metadata.has_author(pattern),
                should_match,
                "Pattern '{}' in content",
                pattern
            );
        }
    }
}

#[test]
fn has_author_special_cases() {
    // Test special characters, unicode, YAML/inline precedence, multiple colons, and whitespace
    let test_cases = vec![
        (
            r#"---
author: O'Brien-Smith
---
Content"#,
            vec![
                ("o'brien", true),
                ("brien-smith", true),
                ("o'brien-smith", true),
            ],
        ),
        (
            r#"---
author: YAML Author
---
author: Inline Author"#,
            vec![("yaml", true), ("inline", true)],
        ),
        (
            r#"author: Name: With: Colons"#,
            vec![("name", true), ("with", true), ("colons", true)],
        ),
        (
            r#"---
author: 张三
---
Content"#,
            vec![("张三", true), ("张", true)],
        ),
        (
            r#"---
author: "  Padded Name  "
---"#,
            vec![("padded", true), ("name", true)],
        ),
    ];

    for (content, expectations) in test_cases {
        let path = PathBuf::from("test.md");
        let metadata = Metadata {
            frontmatter: extract_frontmatter(content, &path),
            raw_content: content.to_string(),
        };

        for (pattern, should_match) in expectations {
            assert_eq!(
                metadata.has_author(pattern),
                should_match,
                "Pattern '{}' failed",
                pattern
            );
        }
    }
}

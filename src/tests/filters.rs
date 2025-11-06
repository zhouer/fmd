use crate::*;
use std::path::PathBuf;

#[test]
fn test_compiled_filters_tag_regex() {
    let args = Args {
        tags: vec!["rust".to_string(), "python".to_string()],
        titles: vec![],
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

    let filters = CompiledFilters::from_args(&args).unwrap();
    assert_eq!(filters.tag_patterns.len(), 2);

    // Check that regex matches work
    let (pattern, regex) = &filters.tag_patterns[0];
    assert_eq!(pattern, "rust");
    assert!(regex.is_match("#rust"));
    assert!(regex.is_match("#RUST")); // case insensitive
    assert!(!regex.is_match("#rust123")); // word boundary
}

#[test]
fn test_compiled_filters_invalid_regex_returns_error() {
    let args = Args {
        tags: vec![],
        titles: vec![],
        authors: vec![],
        names: vec!["[invalid".to_string()], // Invalid regex
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

    let result = CompiledFilters::from_args(&args);
    assert!(result.is_err());
}

#[test]
fn test_compiled_filters_field_validation_errors() {
    // Test various field validation errors with table-driven approach
    let test_cases = vec![
        (":pattern", "Field name cannot be empty"),
        ("field:", "Pattern cannot be empty"),
        (":", "Both field and pattern cannot be empty"),
        ("  :pattern", "Field name cannot be empty"),
    ];

    for (field_input, expected_error) in test_cases {
        let args = Args {
            tags: vec![],
            titles: vec![],
            authors: vec![],
            names: vec![],
            fields: vec![field_input.to_string()],
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

        let result = CompiledFilters::from_args(&args);
        assert!(result.is_err(), "Expected error for input: {}", field_input);
        if let Err(e) = result {
            assert!(
                e.to_string().contains(expected_error),
                "Expected '{}' for input '{}', got '{}'",
                expected_error,
                field_input,
                e
            );
        }
    }
}

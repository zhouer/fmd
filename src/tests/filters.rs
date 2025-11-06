use crate::*;
use std::path::PathBuf;

#[test]
fn test_compiled_filters_tag_regex() {
    let args = Args {
        tags: vec!["rust".to_string(), "python".to_string()],
        titles: vec![],
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
fn test_compiled_filters_empty_field_name() {
    let args = Args {
        tags: vec![],
        titles: vec![],
        names: vec![],
        fields: vec![":pattern".to_string()], // Empty field name
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
    if let Err(e) = result {
        assert!(e.to_string().contains("Field name cannot be empty"));
    }
}

#[test]
fn test_compiled_filters_empty_pattern() {
    let args = Args {
        tags: vec![],
        titles: vec![],
        names: vec![],
        fields: vec!["field:".to_string()], // Empty pattern
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
    if let Err(e) = result {
        assert!(e.to_string().contains("Pattern cannot be empty"));
    }
}

#[test]
fn test_compiled_filters_empty_field_and_pattern() {
    let args = Args {
        tags: vec![],
        titles: vec![],
        names: vec![],
        fields: vec![":".to_string()], // Both empty
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
    if let Err(e) = result {
        assert!(e.to_string().contains("Both field and pattern cannot be empty"));
    }
}

#[test]
fn test_compiled_filters_whitespace_only_field() {
    let args = Args {
        tags: vec![],
        titles: vec![],
        names: vec![],
        fields: vec!["  :pattern".to_string()], // Whitespace-only field
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
    if let Err(e) = result {
        assert!(e.to_string().contains("Field name cannot be empty"));
    }
}

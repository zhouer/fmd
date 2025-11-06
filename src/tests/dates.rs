use crate::*;
use chrono::NaiveDate;
use std::path::PathBuf;

#[test]
fn test_extract_dates_from_frontmatter() {
    let content = "---\ndate: 2025-01-15\ncreated: 2025-01-10\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    let dates = metadata.extract_dates();
    assert_eq!(dates.len(), 2);
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()));
}

#[test]
fn test_extract_dates_from_inline() {
    let content = "# Title\n\ndate: 2025-01-15\nupdated: 2025-01-20";
    let metadata = Metadata {
        frontmatter: None,
        raw_content: content.to_string(),
    };

    let dates = metadata.extract_dates();
    assert_eq!(dates.len(), 2);
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2025, 1, 20).unwrap()));
}

#[test]
fn test_matches_date_filters_after() {
    let content = "---\ndate: 2025-01-15\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    // Should match: date is after 2025-01-10
    assert!(
        metadata.matches_date_filters(Some(NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()), None)
    );

    // Should not match: date is before 2025-01-20
    assert!(
        !metadata.matches_date_filters(Some(NaiveDate::from_ymd_opt(2025, 1, 20).unwrap()), None)
    );
}

#[test]
fn test_matches_date_filters_before() {
    let content = "---\ndate: 2025-01-15\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    // Should match: date is before 2025-01-20
    assert!(
        metadata.matches_date_filters(None, Some(NaiveDate::from_ymd_opt(2025, 1, 20).unwrap()))
    );

    // Should not match: date is after 2025-01-10
    assert!(
        !metadata.matches_date_filters(None, Some(NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()))
    );
}

#[test]
fn test_matches_date_filters_range() {
    let content = "---\ndate: 2025-01-15\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    // Should match: date is in range [2025-01-10, 2025-01-20]
    assert!(metadata.matches_date_filters(
        Some(NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()),
        Some(NaiveDate::from_ymd_opt(2025, 1, 20).unwrap())
    ));

    // Should not match: date is outside range [2025-01-01, 2025-01-10]
    assert!(!metadata.matches_date_filters(
        Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2025, 1, 10).unwrap())
    ));
}

#[test]
fn test_matches_date_filters_multiple_dates() {
    let content = "---\ndate: 2025-01-15\ncreated: 2025-01-05\n---\n# Content";
    let path = PathBuf::from("test.md");
    let fm = extract_frontmatter(content, &path);
    let metadata = Metadata {
        frontmatter: fm,
        raw_content: content.to_string(),
    };

    // Should match: at least one date (created: 2025-01-05) is after 2025-01-01
    assert!(metadata.matches_date_filters(Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()), None));

    // Should match: at least one date (date: 2025-01-15) is in range
    assert!(metadata.matches_date_filters(
        Some(NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()),
        Some(NaiveDate::from_ymd_opt(2025, 1, 20).unwrap())
    ));
}

#[test]
fn test_parse_date_from_yaml_value() {
    let date_value = serde_yaml::Value::String("2025-01-15".to_string());
    let date = parse_date_from_yaml_value(&date_value);
    assert_eq!(date, Some(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()));

    let invalid_value = serde_yaml::Value::String("invalid-date".to_string());
    let date = parse_date_from_yaml_value(&invalid_value);
    assert_eq!(date, None);

    let number_value = serde_yaml::Value::Number(serde_yaml::Number::from(123));
    let date = parse_date_from_yaml_value(&number_value);
    assert_eq!(date, None);
}

#[test]
fn test_date_filter_args_parsing() {
    let args = Args {
        tags: vec![],
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
        date_after: Some("2025-01-01".to_string()),
        date_before: Some("2025-12-31".to_string()),
        dirs: vec![PathBuf::from(".")],
    };

    let filters = CompiledFilters::from_args(&args).unwrap();
    assert_eq!(
        filters.date_after,
        Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    );
    assert_eq!(
        filters.date_before,
        Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
    );
}

#[test]
fn test_date_filter_invalid_format() {
    let args = Args {
        tags: vec![],
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
        date_after: Some("2025/01/01".to_string()), // Invalid format
        date_before: None,
        dirs: vec![PathBuf::from(".")],
    };

    let result = CompiledFilters::from_args(&args);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Invalid date format"));
    }
}

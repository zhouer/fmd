use crate::yaml_value_contains;
use serde_yaml::Value;

#[test]
fn yaml_value_string_positive_cases() {
    // Test exact, case-insensitive, partial matches, and unicode
    let test_cases: Vec<(Value, Vec<(&str, bool)>)> = vec![
        (
            Value::String("hello world".to_string()),
            vec![
                ("hello", true),
                ("world", true),
                ("hello world", true),
                ("hello", true),  // case insensitive
                ("goodbye", false),
            ],
        ),
        (
            Value::String("testing".to_string()),
            vec![("test", true), ("ing", true), ("sti", true)],
        ),
        (
            Value::String("你好世界".to_string()),
            vec![("你好", true), ("世界", true), ("你好世界", true)],
        ),
        (
            Value::String("hello-world_test!@#".to_string()),
            vec![
                ("hello-world", true),
                ("world_test", true),
                ("!@#", true),
            ],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(
                yaml_value_contains(&value, pattern),
                should_match,
                "Pattern '{}' failed",
                pattern
            );
        }
    }
}

#[test]
fn yaml_value_string_negative_cases() {
    let test_cases = vec![
        (
            Value::String("hello world".to_string()),
            vec![("goodbye", false), ("xyz", false)],
        ),
        (
            Value::String("".to_string()),
            vec![("", true), ("something", false)],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(yaml_value_contains(&value, pattern), should_match);
        }
    }
}

#[test]
fn yaml_value_number_types() {
    let test_cases = vec![
        (
            Value::Number(serde_yaml::Number::from(42)),
            vec![("42", true), ("4", true), ("2", true), ("43", false)],
        ),
        (
            Value::Number(serde_yaml::Number::from(3.14f64)),
            vec![("3.14", true), ("3", true), ("14", true), ("15", false)],
        ),
        (
            Value::Number(serde_yaml::Number::from(-100)),
            vec![("-100", true), ("100", true), ("-", true)],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(yaml_value_contains(&value, pattern), should_match);
        }
    }
}

#[test]
fn yaml_value_boolean_types() {
    let test_cases = vec![
        (
            Value::Bool(true),
            vec![
                ("true", true),
                ("tru", true),
                ("true", true),  // case insensitive
                ("false", false),
            ],
        ),
        (
            Value::Bool(false),
            vec![
                ("false", true),
                ("fal", true),
                ("false", true),  // case insensitive
                ("true", false),
            ],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(yaml_value_contains(&value, pattern), should_match);
        }
    }
}

#[test]
fn yaml_value_sequence_positive_cases() {
    let test_cases = vec![
        (
            Value::Sequence(vec![
                Value::String("apple".to_string()),
                Value::String("banana".to_string()),
                Value::String("cherry".to_string()),
            ]),
            vec![("apple", true), ("banana", true), ("cherry", true)],
        ),
        (
            Value::Sequence(vec![
                Value::String("Rust".to_string()),
                Value::String("Python".to_string()),
            ]),
            vec![
                ("rust", true),
                ("python", true),  // case insensitive
                ("python", true),  // case insensitive
            ],
        ),
        (
            Value::Sequence(vec![
                Value::String("testing".to_string()),
                Value::String("development".to_string()),
            ]),
            vec![("test", true), ("dev", true), ("ment", true)],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(yaml_value_contains(&value, pattern), should_match);
        }
    }
}

#[test]
fn yaml_value_sequence_negative_cases() {
    let test_cases = vec![
        (
            Value::Sequence(vec![
                Value::String("one".to_string()),
                Value::String("two".to_string()),
            ]),
            vec![("three", false), ("xyz", false)],
        ),
        (
            Value::Sequence(vec![]),
            vec![("anything", false), ("", false)],
        ),
    ];

    for (value, expectations) in test_cases {
        for (pattern, should_match) in expectations {
            assert_eq!(yaml_value_contains(&value, pattern), should_match);
        }
    }
}

#[test]
fn yaml_value_sequence_mixed_types() {
    let value = Value::Sequence(vec![
        Value::String("text".to_string()),
        Value::Number(serde_yaml::Number::from(123)),
        Value::Bool(true),
    ]);

    assert!(yaml_value_contains(&value, "text"));
    assert!(yaml_value_contains(&value, "123"));
    assert!(yaml_value_contains(&value, "true"));
}

#[test]
fn yaml_value_sequence_with_numbers() {
    let value = Value::Sequence(vec![
        Value::Number(serde_yaml::Number::from(1)),
        Value::Number(serde_yaml::Number::from(2)),
        Value::Number(serde_yaml::Number::from(3)),
    ]);

    assert!(yaml_value_contains(&value, "1"));
    assert!(yaml_value_contains(&value, "2"));
    assert!(yaml_value_contains(&value, "3"));
    assert!(!yaml_value_contains(&value, "4"));
}

#[test]
fn yaml_value_sequence_nested() {
    let value = Value::Sequence(vec![
        Value::Sequence(vec![Value::String("nested".to_string())]),
        Value::String("top".to_string()),
    ]);

    // Should match nested strings through recursion
    assert!(yaml_value_contains(&value, "nested"));
    assert!(yaml_value_contains(&value, "top"));
}

#[test]
fn yaml_value_unsupported_types() {
    // Test null, mapping, and tagged values (all should return false)

    // Null
    let null_value = Value::Null;
    assert!(!yaml_value_contains(&null_value, "null"));
    assert!(!yaml_value_contains(&null_value, ""));
    assert!(!yaml_value_contains(&null_value, "anything"));

    // Mapping
    use serde_yaml::Mapping;
    let mut map = Mapping::new();
    map.insert(
        Value::String("key".to_string()),
        Value::String("value".to_string()),
    );
    let mapping_value = Value::Mapping(map);
    assert!(!yaml_value_contains(&mapping_value, "key"));
    assert!(!yaml_value_contains(&mapping_value, "value"));

    // Tagged
    let tagged_value = Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
        tag: serde_yaml::value::Tag::new("!custom"),
        value: Value::String("tagged".to_string()),
    }));
    assert!(!yaml_value_contains(&tagged_value, "tagged"));
}

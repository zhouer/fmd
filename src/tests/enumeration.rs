use crate::{enumerate_files, Args};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_temp_test_dir() -> (TempDir, PathBuf) {
    // Create temp directory in current dir instead of /tmp
    // This ensures it's within the project and not filtered by ignore rules
    let temp_dir = tempfile::Builder::new()
        .prefix("fmd_test_")
        .tempdir_in(".")
        .unwrap();
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

fn create_test_args(dirs: Vec<PathBuf>, glob: String, depth: Option<usize>) -> Args {
    Args {
        dirs,
        glob,
        tags: vec![],
        titles: vec![],
        authors: vec![],
        fields: vec![],
        names: vec![],
        date_after: None,
        date_before: None,
        full_text: false,
        nul: false,
        ignore_case: false,
        depth,
        verbose: false,
        head_lines: 10,
    }
}

#[test]
fn enumerate_files_basic_glob() {
    let (_temp, temp_path) = create_temp_test_dir();

    // Create test files
    fs::write(temp_path.join("test1.md"), "content").unwrap();
    fs::write(temp_path.join("test2.md"), "content").unwrap();
    fs::write(temp_path.join("test.txt"), "content").unwrap();

    let args = create_test_args(vec![temp_path.clone()], "*.md".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test1.md"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "test2.md"));
}

#[test]
fn enumerate_files_recursive_glob() {
    let (_temp, temp_path) = create_temp_test_dir();

    // Create nested structure
    fs::create_dir(temp_path.join("subdir")).unwrap();
    fs::write(temp_path.join("root.md"), "content").unwrap();
    fs::write(temp_path.join("subdir").join("nested.md"), "content").unwrap();

    let args = create_test_args(vec![temp_path.clone()], "**/*.md".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "root.md"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "nested.md"));
}

#[test]
fn enumerate_files_depth_limits() {
    let (_temp, temp_path) = create_temp_test_dir();

    // Create nested structure
    fs::create_dir(temp_path.join("level1")).unwrap();
    fs::create_dir(temp_path.join("level1").join("level2")).unwrap();

    fs::write(temp_path.join("root.md"), "content").unwrap();
    fs::write(temp_path.join("level1").join("l1.md"), "content").unwrap();
    fs::write(temp_path.join("level1").join("level2").join("l2.md"), "content").unwrap();

    // Test depth=2 (root + 1 level)
    let args = create_test_args(vec![temp_path.clone()], "**/*.md".to_string(), Some(2));
    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "root.md"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "l1.md"));
    assert!(!files.iter().any(|f| f.file_name().unwrap() == "l2.md"));

    // Test depth=1 (only root)
    let args = create_test_args(vec![temp_path.clone()], "**/*.md".to_string(), Some(1));
    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "root.md"));
}

#[test]
fn enumerate_files_excluded_directories() {
    // Test that all common build/cache directories are excluded
    let excluded_dirs = vec![
        "target", "node_modules", "build", "__pycache__", "dist", ".cache"
    ];

    for excluded_dir in excluded_dirs {
        let (_temp, temp_path) = create_temp_test_dir();

        fs::create_dir(temp_path.join(excluded_dir)).unwrap();
        fs::write(temp_path.join("normal.md"), "content").unwrap();
        fs::write(temp_path.join(excluded_dir).join("excluded.md"), "content").unwrap();

        let args = create_test_args(vec![temp_path.clone()], "**/*.md".to_string(), None);

        let files = enumerate_files(&args).unwrap();
        assert_eq!(files.len(), 1, "Failed for directory: {}", excluded_dir);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "normal.md"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "excluded.md"));
    }
}

#[test]
fn enumerate_files_multiple_directories() {
    let (_temp1, temp_path1) = create_temp_test_dir();
    let (_temp2, temp_path2) = create_temp_test_dir();

    fs::write(temp_path1.join("file1.md"), "content").unwrap();
    fs::write(temp_path2.join("file2.md"), "content").unwrap();

    let args = create_test_args(vec![temp_path1.clone(), temp_path2.clone()], "*.md".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.md"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.md"));
}

#[test]
fn enumerate_files_custom_glob_pattern() {
    let (_temp, temp_path) = create_temp_test_dir();

    fs::write(temp_path.join("doc.md"), "content").unwrap();
    fs::write(temp_path.join("note.markdown"), "content").unwrap();
    fs::write(temp_path.join("file.txt"), "content").unwrap();

    let args = create_test_args(vec![temp_path.clone()], "*.markdown".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "note.markdown"));
}

#[test]
fn enumerate_files_specific_path_glob() {
    let (_temp, temp_path) = create_temp_test_dir();

    fs::create_dir(temp_path.join("docs")).unwrap();
    fs::create_dir(temp_path.join("notes")).unwrap();

    fs::write(temp_path.join("docs").join("api.md"), "content").unwrap();
    fs::write(temp_path.join("notes").join("personal.md"), "content").unwrap();

    let args = create_test_args(vec![temp_path.clone()], "**/docs/*.md".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "api.md"));
}

#[test]
fn enumerate_files_invalid_glob_pattern() {
    let (_temp, temp_path) = create_temp_test_dir();

    let args = create_test_args(vec![temp_path.clone()], "[invalid".to_string(), None);

    let result = enumerate_files(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid glob pattern"));
}

#[test]
fn enumerate_files_empty_directory() {
    let (_temp, temp_path) = create_temp_test_dir();

    let args = create_test_args(vec![temp_path.clone()], "*.md".to_string(), None);

    let files = enumerate_files(&args).unwrap();
    assert_eq!(files.len(), 0);
}

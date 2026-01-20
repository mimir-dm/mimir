//! Log viewer command tests.
//!
//! Tests for log file listing, reading, and tailing operations.

use super::common::{create_test_log_file, TestEnv};
use std::fs;

#[tokio::test]
async fn test_logs_directory_setup() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Verify logs directory exists
    assert!(env.paths.logs_dir.exists(), "Logs directory should exist");

    // Chat logs subdirectory should also exist
    let chat_logs = env.paths.logs_dir.join("chat_sessions");
    assert!(chat_logs.exists(), "Chat logs subdirectory should exist");
}

#[tokio::test]
async fn test_create_and_read_log_file() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a test log file
    let log_content = "2024-01-01 INFO Test log entry\n2024-01-01 DEBUG Another entry\n";
    let log_path = create_test_log_file(&env.paths.logs_dir, "test.log", log_content)
        .expect("Failed to create test log");

    // Verify the file was created
    assert!(log_path.exists(), "Log file should exist");

    // Read it back
    let content = fs::read_to_string(&log_path).expect("Failed to read log file");
    assert_eq!(content, log_content);
}

#[tokio::test]
async fn test_list_log_files() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create some test log files
    create_test_log_file(&env.paths.logs_dir, "mimir.log", "Log 1\n")
        .expect("Failed to create log 1");
    create_test_log_file(&env.paths.logs_dir, "mimir.2024-01-01.log", "Log 2\n")
        .expect("Failed to create log 2");

    // List log files
    let entries = fs::read_dir(&env.paths.logs_dir)
        .expect("Failed to read logs dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect::<Vec<_>>();

    assert!(entries.len() >= 2, "Should have at least 2 log files");
}

#[tokio::test]
async fn test_log_file_with_multiple_lines() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a log file with many lines
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!("Line {}: Test log entry\n", i));
    }

    let log_path = create_test_log_file(&env.paths.logs_dir, "large.log", &content)
        .expect("Failed to create large log");

    // Verify line count
    let read_content = fs::read_to_string(&log_path).expect("Failed to read log");
    let line_count = read_content.lines().count();
    assert_eq!(line_count, 100, "Should have 100 lines");
}

#[tokio::test]
async fn test_paths_accessible_from_app_state() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // This is the key test - verify that paths are accessible through state
    // The same way Tauri commands would access them
    let logs_dir = &env.state.paths.logs_dir;
    assert!(logs_dir.exists(), "Logs dir should be accessible via state.paths");

    let data_dir = &env.state.paths.data_dir;
    assert!(data_dir.exists(), "Data dir should be accessible via state.paths");
}

//! Book content command tests.
//!
//! Tests for book content retrieval, image serving, and reference lookup.

use super::common::{create_test_book, TestEnv};
use std::fs;

#[tokio::test]
async fn test_books_directory_setup() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Verify books directory exists
    let books_dir = env.paths.data_dir.join("books");
    assert!(books_dir.exists(), "Books directory should exist");
}

#[tokio::test]
async fn test_create_test_book() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a test book
    let book_dir = create_test_book(&env.paths.data_dir, "TEST-BOOK")
        .expect("Failed to create test book");

    // Verify structure
    assert!(book_dir.exists(), "Book directory should exist");
    assert!(book_dir.join("book").exists(), "Book content directory should exist");

    // Verify book JSON file exists
    let book_json = book_dir.join("book").join("book-test-book.json");
    assert!(book_json.exists(), "Book JSON file should exist");

    // Verify JSON content
    let content = fs::read_to_string(&book_json).expect("Failed to read book JSON");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    assert_eq!(json["name"], "TEST-BOOK");
    assert_eq!(json["id"], "TEST-BOOK");
}

#[tokio::test]
async fn test_find_book_content_file() {
    use mimir_dm::commands::content::books::book_content::find_book_content_file;

    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a test book
    let book_dir = create_test_book(&env.paths.data_dir, "FIND-TEST")
        .expect("Failed to create test book");

    // Use the actual find function
    let result = find_book_content_file(&book_dir);
    assert!(result.is_ok(), "Should not error when searching for book content");

    let content_path = result.unwrap();
    assert!(content_path.is_some(), "Should find book content file");

    let path = content_path.unwrap();
    assert!(path.exists(), "Found path should exist");
    assert!(path.to_string_lossy().contains("book-find-test.json"));
}

#[tokio::test]
async fn test_book_not_found() {
    use mimir_dm::commands::content::books::book_content::find_book_content_file;

    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create an empty book directory (no content file)
    let empty_book = env.paths.data_dir.join("books").join("EMPTY-BOOK");
    fs::create_dir_all(&empty_book).expect("Failed to create empty book dir");

    // Should return None (not an error)
    let result = find_book_content_file(&empty_book);
    assert!(result.is_ok(), "Should not error for missing content");
    assert!(result.unwrap().is_none(), "Should return None when no content found");
}

#[tokio::test]
async fn test_multiple_books() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create multiple test books
    create_test_book(&env.paths.data_dir, "PHB")
        .expect("Failed to create PHB");
    create_test_book(&env.paths.data_dir, "DMG")
        .expect("Failed to create DMG");
    create_test_book(&env.paths.data_dir, "MM")
        .expect("Failed to create MM");

    // List books
    let books_dir = env.paths.data_dir.join("books");
    let entries: Vec<_> = fs::read_dir(&books_dir)
        .expect("Failed to read books dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    assert_eq!(entries.len(), 3, "Should have 3 books");
}

#[tokio::test]
async fn test_book_with_images_directory() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a book with images
    let book_dir = create_test_book(&env.paths.data_dir, "IMG-TEST")
        .expect("Failed to create test book");

    // Create images directory
    let img_dir = book_dir.join("img").join("book").join("IMG-TEST");
    fs::create_dir_all(&img_dir).expect("Failed to create img directory");

    // Create a dummy image file
    fs::write(img_dir.join("test.png"), b"fake image data")
        .expect("Failed to create test image");

    // Verify structure
    assert!(img_dir.join("test.png").exists(), "Image file should exist");
}

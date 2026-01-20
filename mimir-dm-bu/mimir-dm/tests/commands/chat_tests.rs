//! Chat session command tests.
//!
//! Tests for chat session CRUD operations and session management.

use super::common::TestEnv;
use mimir_dm::commands::chat::chat_sessions::{ChatMessage, ChatSession, SessionManager};

#[tokio::test]
async fn test_app_state_construction() {
    // Verify we can construct a valid AppState
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Verify paths are set correctly
    assert!(env.paths.data_dir.exists(), "Data dir should exist");
    assert!(env.paths.logs_dir.exists(), "Logs dir should exist");
    assert!(env.paths.config_dir.exists(), "Config dir should exist");
}

#[tokio::test]
async fn test_session_manager_initialization() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // The session manager should be created during TestEnv::new()
    // Verify the sessions directory exists
    let sessions_dir = env.paths.data_dir.join("chat_sessions");
    assert!(sessions_dir.exists(), "Chat sessions directory should exist");
}

#[tokio::test]
async fn test_create_and_list_sessions() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Create a session manager directly
    let session_manager = SessionManager::new(&env.paths)
        .expect("Failed to create session manager");

    // Initially there should be no sessions
    let sessions = session_manager.list_sessions().expect("Failed to list sessions");
    assert!(sessions.is_empty(), "No sessions should exist initially");

    // Create a test session
    let session = ChatSession {
        id: "test-session-1".to_string(),
        title: "Test Session".to_string(),
        created_at: 1700000000,
        updated_at: 1700000000,
        messages: vec![
            ChatMessage {
                id: "msg-1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                timestamp: 1700000000,
                token_usage: None,
            },
        ],
    };

    // Save the session
    session_manager.save_session(session).expect("Failed to save session");

    // List sessions should now return one
    let sessions = session_manager.list_sessions().expect("Failed to list sessions");
    assert_eq!(sessions.len(), 1, "Should have one session");
    assert_eq!(sessions[0].id, "test-session-1");
    assert_eq!(sessions[0].title, "Test Session");
}

#[tokio::test]
async fn test_load_session() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let session_manager = SessionManager::new(&env.paths)
        .expect("Failed to create session manager");

    // Create and save a session
    let session = ChatSession {
        id: "test-session-load".to_string(),
        title: "Load Test".to_string(),
        created_at: 1700000000,
        updated_at: 1700000000,
        messages: vec![
            ChatMessage {
                id: "msg-1".to_string(),
                role: "user".to_string(),
                content: "Test message".to_string(),
                timestamp: 1700000000,
                token_usage: None,
            },
        ],
    };

    // Save expected values before moving
    let expected_id = session.id.clone();
    let expected_title = session.title.clone();

    session_manager.save_session(session).expect("Failed to save session");

    // Load it back
    let loaded = session_manager.load_session("test-session-load")
        .expect("Failed to load session")
        .expect("Session should exist");

    assert_eq!(loaded.id, expected_id);
    assert_eq!(loaded.title, expected_title);
    assert_eq!(loaded.messages.len(), 1);
    assert_eq!(loaded.messages[0].content, "Test message");
}

#[tokio::test]
async fn test_delete_session() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let session_manager = SessionManager::new(&env.paths)
        .expect("Failed to create session manager");

    // Create and save a session
    let session = ChatSession {
        id: "test-session-delete".to_string(),
        title: "Delete Test".to_string(),
        created_at: 1700000000,
        updated_at: 1700000000,
        messages: vec![],
    };

    session_manager.save_session(session).expect("Failed to save session");

    // Verify it exists
    let sessions = session_manager.list_sessions().expect("Failed to list sessions");
    assert_eq!(sessions.len(), 1);

    // Delete it
    session_manager.delete_session("test-session-delete")
        .expect("Failed to delete session");

    // Verify it's gone
    let sessions = session_manager.list_sessions().expect("Failed to list sessions");
    assert!(sessions.is_empty(), "Session should be deleted");
}

#[tokio::test]
async fn test_database_service_available() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Verify we can get a database connection
    let conn = env.state.db.get_connection();
    assert!(conn.is_ok(), "Should be able to get database connection");
}

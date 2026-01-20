//! Chat-specific logging system
//!
//! Provides session-based logging for chat interactions, allowing separate
//! log files per chat session for easier analysis.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Token usage information for chat logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

/// Chat log event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ChatLogEvent {
    /// User message received
    UserMessage {
        content: String,
        message_id: Option<String>,
    },
    /// System prompt injected
    SystemPrompt {
        content: String,
        context_type: String,
    },
    /// LLM call started
    LlmCall {
        iteration: usize,
        messages_count: usize,
        tools_enabled: bool,
        model: String,
    },
    /// LLM response received
    LlmResponse {
        content: String,
        tokens: Option<ChatTokenUsage>,
        tool_calls_count: usize,
    },
    /// Tool call executed
    ToolCall {
        tool_name: String,
        args: Value,
        success: bool,
        result: String,
        execution_time_ms: Option<u64>,
    },
    /// Chat session error
    Error {
        context: String,
        error: String,
        error_type: String,
    },
    /// Session metadata
    SessionInfo { action: String, details: Value },
    /// Complete conversation context sent to LLM
    FullConversationContext {
        iteration: usize,
        messages: Vec<Value>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        tools_enabled: bool,
        tools_count: usize,
    },
}

/// Structured chat log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_id: String,
    #[serde(flatten)]
    pub event: ChatLogEvent,
}

/// Chat logger for a specific session
pub struct ChatLogger {
    session_id: String,
    log_file_path: PathBuf,
}

impl ChatLogger {
    /// Create a new chat logger for a session
    pub fn new(session_id: String, logs_dir: &Path) -> Result<Self> {
        // Create chat_sessions subdirectory if it doesn't exist
        let chat_logs_dir = logs_dir.join("chat_sessions");
        fs::create_dir_all(&chat_logs_dir).with_context(|| {
            format!(
                "Failed to create chat logs directory: {}",
                chat_logs_dir.display()
            )
        })?;

        let sanitized_session_id = sanitize_filename(&session_id);
        let log_file_path = chat_logs_dir.join(format!("{}.log", sanitized_session_id));

        info!(
            "Created chat logger for session: {} -> {}",
            session_id,
            log_file_path.display()
        );

        Ok(Self {
            session_id,
            log_file_path,
        })
    }

    /// Log a structured chat event
    pub fn log_event(&self, event: ChatLogEvent) {
        let entry = ChatLogEntry {
            timestamp: chrono::Utc::now(),
            session_id: self.session_id.clone(),
            event,
        };

        match serde_json::to_string(&entry) {
            Ok(json_line) => {
                // Write directly to the log file
                match OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.log_file_path)
                {
                    Ok(mut file) => {
                        if let Err(e) = writeln!(file, "{}", json_line) {
                            error!("Failed to write to chat log: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to open chat log file: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize chat log entry: {}", e);
            }
        }
    }

    /// Log user message
    pub fn log_user_message(&self, content: &str, message_id: Option<String>) {
        self.log_event(ChatLogEvent::UserMessage {
            content: content.to_string(),
            message_id,
        });
    }

    /// Log system prompt
    pub fn log_system_prompt(&self, content: &str, context_type: &str) {
        self.log_event(ChatLogEvent::SystemPrompt {
            content: content.to_string(),
            context_type: context_type.to_string(),
        });
    }

    /// Log LLM call
    pub fn log_llm_call(
        &self,
        iteration: usize,
        messages_count: usize,
        tools_enabled: bool,
        model: &str,
    ) {
        self.log_event(ChatLogEvent::LlmCall {
            iteration,
            messages_count,
            tools_enabled,
            model: model.to_string(),
        });
    }

    /// Log LLM response
    pub fn log_llm_response(
        &self,
        content: &str,
        tokens: Option<ChatTokenUsage>,
        tool_calls_count: usize,
    ) {
        self.log_event(ChatLogEvent::LlmResponse {
            content: content.to_string(),
            tokens,
            tool_calls_count,
        });
    }

    /// Log tool call
    pub fn log_tool_call(
        &self,
        tool_name: &str,
        args: &Value,
        success: bool,
        result: &str,
        execution_time_ms: Option<u64>,
    ) {
        self.log_event(ChatLogEvent::ToolCall {
            tool_name: tool_name.to_string(),
            args: args.clone(),
            success,
            result: result.to_string(),
            execution_time_ms,
        });
    }

    /// Log error
    pub fn log_error(&self, context: &str, error: &str, error_type: &str) {
        self.log_event(ChatLogEvent::Error {
            context: context.to_string(),
            error: error.to_string(),
            error_type: error_type.to_string(),
        });
    }

    /// Log session info
    pub fn log_session_info(&self, action: &str, details: Value) {
        self.log_event(ChatLogEvent::SessionInfo {
            action: action.to_string(),
            details,
        });
    }

    /// Log complete conversation context sent to LLM
    pub fn log_full_conversation_context(
        &self,
        iteration: usize,
        messages: &[mimir_dm_llm::Message],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        tools_enabled: bool,
        tools_count: usize,
    ) {
        let message_values: Vec<Value> = messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role,
                    "content": msg.content
                })
            })
            .collect();

        self.log_event(ChatLogEvent::FullConversationContext {
            iteration,
            messages: message_values,
            temperature,
            max_tokens,
            tools_enabled,
            tools_count,
        });
    }

    /// Get the session ID
    #[cfg(test)]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the log file path
    #[cfg(test)]
    pub fn log_file_path(&self) -> &Path {
        &self.log_file_path
    }
}

/// Sanitize a filename to be safe for filesystem use
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Replace unsafe characters with underscores
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            // Keep alphanumeric, hyphens, and underscores
            c if c.is_alphanumeric() || c == '-' || c == '_' => c,
            // Replace other characters with underscores
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("abc123"), "abc123");
        assert_eq!(sanitize_filename("abc-123_def"), "abc-123_def");
        assert_eq!(sanitize_filename("abc/def\\ghi:jkl"), "abc_def_ghi_jkl");
        assert_eq!(
            sanitize_filename("session-id-with-special-chars!@#"),
            "session-id-with-special-chars___"
        );
    }

    #[test]
    fn test_chat_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let session_id = "test-session-123".to_string();

        let logger = ChatLogger::new(session_id.clone(), temp_dir.path()).unwrap();

        assert_eq!(logger.session_id(), &session_id);
        assert!(
            logger.log_file_path().exists() || logger.log_file_path().parent().unwrap().exists()
        );
    }
}

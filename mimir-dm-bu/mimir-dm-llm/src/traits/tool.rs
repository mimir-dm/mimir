//! Tool trait for LLM function calling
//!
//! This module defines the trait that all tools must implement to be callable by the LLM.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::time::Instant;

use super::provider::{Tool as LlmTool, ToolFunction};

/// Represents a recent tool call for context tracking.
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Name of the tool that was called.
    pub name: String,
    /// When the tool was called.
    pub timestamp: Instant,
    /// File path involved in the call, if any.
    pub file_path: Option<String>,
}

/// Risk level for tool actions that modify state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk - e.g., updating documentation
    Low,
    /// Medium risk - e.g., modifying configuration
    Medium,
    /// High risk - e.g., deleting data
    High,
}

/// Description of an action that requires confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDescription {
    /// Brief title of the action
    pub title: String,
    /// Detailed description of what will happen
    pub description: String,
    /// Structured changes that will be made
    pub changes: ChangeDetail,
}

/// Structured representation of changes for frontend rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChangeDetail {
    /// File editing with line-number based changes.
    FileEdit {
        /// Path to the file being edited.
        file_path: String,
        /// List of individual edits to apply.
        edits: Vec<LineEdit>,
        /// Total number of lines affected by edits.
        total_lines_affected: usize,
        /// Total number of lines in the file.
        total_lines_in_file: usize,
    },
    /// File writing with diff preview.
    FileWrite {
        /// Path to the file being written.
        file_path: String,
        /// Length of the content being written.
        content_length: usize,
        /// Diff preview if available.
        diff_preview: Option<DiffPreview>,
        /// Content to write (truncated if too long for preview).
        content_preview: Option<String>,
    },
    /// File reading operation.
    FileRead {
        /// Path to the file being read.
        file_path: String,
        /// Size of the file in bytes.
        file_size: usize,
    },
    /// Generic changes (fallback for other operation types).
    Generic {
        /// List of change descriptions.
        items: Vec<String>,
    },
}

/// Individual line-based edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineEdit {
    /// Type of edit operation
    pub operation: EditOperation,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,
    /// Original content being replaced
    pub old_content: Vec<String>,
    /// New content to insert
    pub new_content: Vec<String>,
    /// Multiple context lines before the edit (for preview)
    pub context_before: Vec<String>,
    /// Multiple context lines after the edit (for preview)
    pub context_after: Vec<String>,
}

/// Type of edit operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditOperation {
    /// Replace existing content with new content.
    Replace,
    /// Insert new content at a position.
    Insert,
    /// Delete existing content.
    Delete,
}

/// Diff preview information showing changes between versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPreview {
    /// Number of lines added.
    pub added_lines: usize,
    /// Number of lines removed.
    pub removed_lines: usize,
    /// Human-readable diff preview text.
    pub preview: String,
}

/// Trait that all callable tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool's unique name
    fn name(&self) -> &str;

    /// Get the tool's description for the LLM
    fn description(&self) -> &str;

    /// Get the JSON Schema for the tool's parameters
    fn parameters_schema(&self) -> Value;

    /// Whether this tool requires user confirmation before execution
    ///
    /// Default implementation returns false (no confirmation needed)
    fn requires_confirmation(&self) -> bool {
        false
    }

    /// Generate a human-readable description of the action for confirmation
    ///
    /// This is only called when `requires_confirmation()` returns true.
    /// Default implementation returns None.
    fn describe_action(&self, _arguments: &Value) -> Option<ActionDescription> {
        None
    }

    /// Execute the tool with the given arguments
    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>>;

    /// Execute with access to recent tool calls (default delegates to execute)
    async fn execute_with_context(
        &self,
        arguments: Value,
        _recent_calls: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<ToolCall>>>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Default implementation ignores context
        self.execute(arguments).await
    }

    /// Get workflow guidance for this tool (relationships, dependencies, usage patterns)
    ///
    /// Default implementation returns None. Tools should override this to provide
    /// guidance about how they interact with other tools.
    fn workflow_guidance(&self) -> Option<String> {
        None
    }

    /// Convert to LLM tool definition
    fn to_llm_tool(&self) -> LlmTool {
        let name = self.name().to_string();
        LlmTool {
            name: name.clone(),
            tool_type: "function".to_string(),
            function: ToolFunction {
                name,
                description: self.description().to_string(),
                parameters: self.parameters_schema(),
            },
        }
    }
}

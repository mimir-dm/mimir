//! Todo List Tool for managing complex task workflows
//!
//! Helps the LLM track multi-step tasks and avoid context rot during long conversations.
//! This tool manages ephemeral todos in memory for the duration of a chat session.

use crate::ToolTrait;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Represents a single todo item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Description of the task.
    pub content: String,
    /// Current status: "pending", "in_progress", or "completed".
    pub status: String,
    /// Present tense form for display during execution.
    #[serde(rename = "activeForm")]
    pub active_form: String,
}

/// Manages todo state with configurable storage backend
#[derive(Debug, Clone)]
pub struct TodoStateManager {
    storage_path: Arc<Mutex<Option<PathBuf>>>,
}

impl Default for TodoStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoStateManager {
    /// Creates a new todo state manager.
    pub fn new() -> Self {
        Self {
            storage_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Configure the storage path for todos (this should be a directory path)
    pub fn configure_storage(&self, path: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Ensure the directory exists (path is the directory, not a file)
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create todos directory: {}", e))?;

        let mut storage_path = self.storage_path.lock().unwrap();
        *storage_path = Some(path);
        debug!(
            "Todo storage configured to: {:?}",
            storage_path.as_ref().unwrap()
        );
        Ok(())
    }

    /// Get the file path for a session's todos
    fn get_session_file_path(
        &self,
        session_id: &str,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let storage_path = self.storage_path.lock().unwrap();
        let base_path = storage_path
            .as_ref()
            .ok_or("Todo storage not configured. Call configure_storage first.")?;
        Ok(base_path.join(format!("{}.json", session_id)))
    }

    /// Get todos for a session
    pub fn get_todos(&self, session_id: &str) -> Vec<TodoItem> {
        match self.load_todos_from_file(session_id) {
            Ok(todos) => todos,
            Err(e) => {
                debug!("Failed to load todos for session {}: {}", session_id, e);
                Vec::new()
            }
        }
    }

    /// Set todos for a session
    pub fn set_todos(
        &self,
        session_id: &str,
        todos: Vec<TodoItem>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.save_todos_to_file(session_id, &todos)
    }

    /// Load todos from file for a session
    fn load_todos_from_file(
        &self,
        session_id: &str,
    ) -> Result<Vec<TodoItem>, Box<dyn Error + Send + Sync>> {
        let file_path = self.get_session_file_path(session_id)?;

        if !file_path.exists() {
            debug!(
                "Todo file does not exist for session {}, returning empty list",
                session_id
            );
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read todo file: {}", e))?;

        let todos: Vec<TodoItem> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse todo JSON: {}", e))?;

        debug!("Loaded {} todos for session {}", todos.len(), session_id);
        Ok(todos)
    }

    /// Save todos to file for a session
    fn save_todos_to_file(
        &self,
        session_id: &str,
        todos: &[TodoItem],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file_path = self.get_session_file_path(session_id)?;

        let json_content = serde_json::to_string_pretty(todos)
            .map_err(|e| format!("Failed to serialize todos: {}", e))?;

        std::fs::write(&file_path, json_content)
            .map_err(|e| format!("Failed to write todo file: {}", e))?;

        info!("Saved {} todos for session {}", todos.len(), session_id);
        Ok(())
    }

    /// Clear todos for a session
    pub fn clear_session(&self, session_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file_path = self.get_session_file_path(session_id)?;
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to remove todo file: {}", e))?;
        }
        Ok(())
    }
}

/// Tool for managing todo lists with in-memory persistence
pub struct TodoListTool {
    state_manager: TodoStateManager,
}

impl TodoListTool {
    /// Creates a new todo list tool with the given state manager.
    pub fn new(state_manager: TodoStateManager) -> Self {
        Self { state_manager }
    }

    /// Validate that only one task can be in_progress at a time
    fn validate_single_in_progress(todos: &[TodoItem]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let in_progress_count = todos
            .iter()
            .filter(|todo| todo.status == "in_progress")
            .count();

        if in_progress_count > 1 {
            return Err("Only one task can be in_progress at a time".into());
        }

        Ok(())
    }
}

#[async_trait]
impl ToolTrait for TodoListTool {
    fn name(&self) -> &str {
        "todo_write"
    }

    fn description(&self) -> &str {
        "Create and manage structured task lists for complex coding sessions. Tracks progress, organizes multi-step tasks, and demonstrates thoroughness to users.

Usage:
- Use for complex tasks requiring 3+ distinct steps or multiple operations
- Mark tasks in_progress BEFORE beginning work (exactly ONE at a time)
- Mark completed IMMEDIATELY after finishing (don't batch)
- Create specific, actionable items with clear descriptions

When to use:
- Complex multi-step tasks requiring careful planning
- User provides multiple tasks or numbered lists
- Non-trivial tasks needing progress tracking
- When starting work on any task (mark in_progress first)

When NOT to use:
- Single straightforward tasks
- Trivial tasks with <3 steps
- Purely conversational or informational requests

Task states:
- pending: Not yet started
- in_progress: Currently working (limit ONE task)
- completed: Finished successfully

Required format:
- content: Imperative form (\"Run tests\", \"Fix bug\")  
- activeForm: Present continuous (\"Running tests\", \"Fixing bug\")
- status: Must be \"status\" field, NOT \"state\"

Example: {\"todos\": [{\"content\": \"Fix auth\", \"status\": \"pending\", \"activeForm\": \"Fixing auth\"}]}

Task management: Update status real-time, complete current before starting new, remove irrelevant tasks entirely."
    }

    fn workflow_guidance(&self) -> Option<String> {
        Some("Use for complex multi-step tasks (3+ steps). Mark ONE task in_progress before starting work, mark completed immediately after finishing.".to_string())
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "The updated todo list",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "Task description in imperative form (e.g., 'Fix bug', 'Run tests')",
                                "minLength": 1
                            },
                            "status": {
                                "type": "string",
                                "description": "Task status - MUST be 'status' field name, NOT 'state'",
                                "enum": ["pending", "in_progress", "completed"]
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "Task description in present continuous form (e.g., 'Fixing bug', 'Running tests')",
                                "minLength": 1
                            }
                        },
                        "required": ["content", "status", "activeForm"],
                        "additionalProperties": false
                    }
                },
                "session_id": {
                    "type": "string",
                    "description": "Chat session ID for todo persistence (optional, defaults to 'default')"
                }
            },
            "required": ["todos"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Accept both "todos" and "tasks" parameter names for backward compatibility
        let todos_array = arguments
            .get("todos")
            .or_else(|| arguments.get("tasks"))
            .and_then(|v| v.as_array())
            .ok_or("Missing or invalid 'todos' parameter")?;

        let todos: Vec<TodoItem> = todos_array
            .iter()
            .map(|item| -> Result<TodoItem, Box<dyn Error + Send + Sync>> {
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'content' field")?;

                let status = item
                    .get("status")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'status' field")?;

                let active_form = item
                    .get("activeForm")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'activeForm' field")?;

                // Validate status
                if !["pending", "in_progress", "completed"].contains(&status) {
                    return Err(format!("Invalid status: {}", status).into());
                }

                Ok(TodoItem {
                    content: content.to_string(),
                    status: status.to_string(),
                    active_form: active_form.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Validate only one in_progress task
        Self::validate_single_in_progress(&todos)?;

        // Get session ID from arguments or use default
        let session_id = arguments
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Store the todos using the configured storage
        self.state_manager.set_todos(session_id, todos.clone())?;

        // Return a summary
        let pending_count = todos.iter().filter(|t| t.status == "pending").count();
        let in_progress_count = todos.iter().filter(|t| t.status == "in_progress").count();
        let completed_count = todos.iter().filter(|t| t.status == "completed").count();

        let summary = if in_progress_count > 0 {
            let current_task = todos
                .iter()
                .find(|t| t.status == "in_progress")
                .map(|t| t.active_form.as_str())
                .unwrap_or("Unknown task");

            format!(
                "Todos have been modified successfully. Ensure that you continue to use the todo list to track your progress. Please proceed with the current tasks if applicable\n\nCurrent status: {} ({} pending, {} in progress, {} completed)",
                current_task, pending_count, in_progress_count, completed_count
            )
        } else {
            format!(
                "Todos have been modified successfully. Ensure that you continue to use the todo list to track your progress. Please proceed with the current tasks if applicable\n\nStatus: {} pending, {} completed",
                pending_count, completed_count
            )
        };

        info!(
            "Updated todo list: {} items total for session {}",
            todos.len(),
            session_id
        );
        debug!("Todo list updated: {:?}", todos);
        Ok(summary)
    }
}

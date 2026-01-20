//! Generic file-based tools for LLM
//!
//! These tools provide basic file operations without any domain knowledge,
//! making them reusable across different applications.

use crate::traits::tool::DiffPreview;
use crate::traits::{
    ActionDescription, ChangeDetail, EditOperation, LineEdit, ToolCallContext as ToolCall,
};
use crate::{FileToolsConfig, ToolTrait};
use async_trait::async_trait;
use serde_json::{json, Value};
use similar::{ChangeTag, TextDiff};
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Path validator for security - ensures file access is restricted to allowed directories
#[derive(Debug, Clone)]
pub struct PathValidator {
    allowed_prefixes: Vec<PathBuf>,
    forbidden_patterns: Vec<String>,
}

impl PathValidator {
    /// Create a new path validator with allowed directory prefixes
    pub fn new(allowed_prefixes: Vec<PathBuf>) -> Self {
        // Canonicalize allowed prefixes so they match canonicalized paths during validation
        let canonical_prefixes: Vec<PathBuf> = allowed_prefixes
            .into_iter()
            .filter_map(|p| p.canonicalize().ok())
            .collect();

        Self {
            allowed_prefixes: canonical_prefixes,
            forbidden_patterns: vec![
                "..".to_string(),
                "/etc/".to_string(),
                "/var/log/".to_string(),
                "/usr/bin/".to_string(),
                "/bin/".to_string(),
                "/sys/".to_string(),
                "/proc/".to_string(),
            ],
        }
    }

    /// Validate that a path is safe to access
    pub fn validate_path(&self, path: &str) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        self.validate_path_internal(path, false)
    }

    /// Validate that a path is safe to access, allowing parent directory creation
    pub fn validate_path_for_write(
        &self,
        path: &str,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        self.validate_path_internal(path, true)
    }

    /// Internal validation method with directory creation control
    fn validate_path_internal(
        &self,
        path: &str,
        allow_dir_creation: bool,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let path = Path::new(path);

        // Check for forbidden patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.forbidden_patterns {
            if path_str.contains(pattern) {
                return Err(format!("Path contains forbidden pattern: {}", pattern).into());
            }
        }

        // Canonicalize the path to resolve any symbolic links or relative components
        let canonical_path = path.canonicalize()
            .or_else(|_| {
                // If canonicalize fails (file doesn't exist), try with parent directory
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        let canonical_parent = parent.canonicalize()?;
                        if let Some(filename) = path.file_name() {
                            Ok(canonical_parent.join(filename))
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))
                        }
                    } else if allow_dir_creation {
                        // For write operations, allow non-existent parent directories 
                        // since WriteFileTool creates them. Just validate the path structure.
                        let _canonical_root = self.allowed_prefixes.iter()
                            .find(|prefix| path.starts_with(prefix))
                            .ok_or_else(|| {
                                let allowed_dirs: Vec<String> = self.allowed_prefixes.iter()
                                    .map(|p| p.display().to_string())
                                    .collect();
                                std::io::Error::new(
                                    std::io::ErrorKind::PermissionDenied,
                                    format!(
                                        "Path '{}' is outside allowed directories. Use absolute paths starting with: {}. Example: '{}/your_filename.txt'",
                                        path.display(),
                                        allowed_dirs.join(" or "),
                                        allowed_dirs.first().unwrap_or(&"[no allowed dirs]".to_string())
                                    )
                                )
                            })?;

                        // Return the path as-is if it's within allowed directories
                        // The parent directory will be created by the tool if needed
                        Ok(path.to_path_buf())
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!(
                                "Parent directory does not exist: {}. Use list_files to check directory structure or use write_file which can create directories.",
                                parent.display()
                            )
                        ))
                    }
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))
                }
            })
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        // Check that the path is within allowed prefixes
        let is_allowed = self
            .allowed_prefixes
            .iter()
            .any(|prefix| canonical_path.starts_with(prefix));

        if !is_allowed {
            let allowed_dirs: Vec<String> = self
                .allowed_prefixes
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            return Err(format!(
                "Path '{}' is not within allowed directories. Use absolute paths starting with: {}. Example: '{}/your_filename.txt'",
                canonical_path.display(),
                allowed_dirs.join(" or "),
                allowed_dirs.first().unwrap_or(&"[no allowed dirs]".to_string())
            ).into());
        }

        Ok(canonical_path)
    }
}

/// Generate a diff display between current and new content
fn generate_diff_display(current_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(current_content, new_content);
    let mut diff_output = String::new();
    let mut line_count = 0;
    const MAX_LINES: usize = 50;

    for change in diff.iter_all_changes() {
        if line_count >= MAX_LINES {
            diff_output.push_str(&format!(
                "\n... ({} more lines changed) ...",
                diff.iter_all_changes().count() - line_count
            ));
            break;
        }

        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };

        // Only show changed lines and minimal context
        match change.tag() {
            ChangeTag::Delete | ChangeTag::Insert => {
                diff_output.push_str(&format!("{}{}", sign, change));
                line_count += 1;
            }
            ChangeTag::Equal => {
                // Show context lines (unchanged lines around changes)
                let line = change.to_string();
                if !line.trim().is_empty() && line.len() < 100 {
                    diff_output.push_str(&format!("{}{}", sign, change));
                    line_count += 1;
                }
            }
        }
    }

    // Check if there are actually any changes (insertions/deletions)
    let has_changes = diff
        .iter_all_changes()
        .any(|change| matches!(change.tag(), ChangeTag::Delete | ChangeTag::Insert));

    if !has_changes || diff_output.trim().is_empty() {
        "No changes detected".to_string()
    } else {
        format!("```diff\n{}\n```", diff_output)
    }
}

/// Tool for reading file contents
pub struct ReadFileTool {
    config: Arc<FileToolsConfig>,
}

impl ReadFileTool {
    /// Creates a new read file tool with the given configuration.
    pub fn new(config: Arc<FileToolsConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ToolTrait for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        // Static fallback description
        "Read the contents of a file and return it with line numbers for easy editing and reference. Use to_llm_tool() for dynamic path-aware description."
    }

    fn workflow_guidance(&self) -> Option<String> {
        Some(
            "ALWAYS use read_file before edit_file to understand current content and structure"
                .to_string(),
        )
    }

    fn to_llm_tool(&self) -> crate::traits::provider::Tool {
        let base_path = self
            .config
            .allowed_directories
            .first()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[no allowed directories configured]".to_string());

        let dynamic_description = format!(
            "Read the contents of a file and return it with line numbers for easy editing and reference.

Usage:
- File paths MUST be absolute paths starting with: {}
- Example: '{}/your_filename.txt'
- Returns content in line-numbered format (e.g., '  1→content')
- Essential prerequisite before using edit_file tool
- Handles text files; binary files may produce garbled output
- Empty files return '(empty file)' message

When to use:
- Before any edit operation to understand current content
- To examine file structure and locate specific lines
- For code review with clear line references

When NOT to use:
- If you know the file doesn't exist (will return an error)
- For binary files (use specialized tools instead)

Security: All file operations restricted to application data directory",
            base_path, base_path
        );

        crate::traits::provider::Tool {
            name: self.name().to_string(),
            tool_type: "function".to_string(),
            function: crate::traits::provider::ToolFunction {
                name: self.name().to_string(),
                description: dynamic_description,
                parameters: self.parameters_schema(),
            },
        }
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' parameter")?;

        // Validate the path using PathValidator
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let validated_path = path_validator.validate_path(file_path)?;

        // Check that it's actually a file
        if !validated_path.is_file() {
            if validated_path.is_dir() {
                return Err(format!(
                    "Path is a directory, not a file: {}. Use list_files to explore directory contents.", 
                    validated_path.display()
                ).into());
            } else {
                return Err(format!(
                    "File does not exist: {}. Use list_files to check the directory structure and verify the correct path.", 
                    validated_path.display()
                ).into());
            }
        }

        // Read the file
        let content = fs::read_to_string(&validated_path)
            .map_err(|e| format!(
                "Failed to read file '{}': {}. Check that the file exists and is readable. Use list_files to verify the file is present.", 
                validated_path.display(), e
            ))?;

        // Format content with line numbers for LLM use
        let lines = content.lines();
        let line_count = lines.clone().count();

        if line_count == 0 {
            debug!("Read empty file: {}", validated_path.display());
            return Ok("(empty file)".to_string());
        }

        // Calculate the width needed for line numbers (minimum 3 digits)
        let line_number_width = std::cmp::max(3, (line_count as f64).log10().floor() as usize + 1);

        let formatted_content = lines
            .enumerate()
            .map(|(i, line)| format!("{:width$}→{}", i + 1, line, width = line_number_width))
            .collect::<Vec<_>>()
            .join("\n");

        debug!(
            "Read file with {} lines: {}",
            line_count,
            validated_path.display()
        );
        Ok(formatted_content)
    }
}

/// Tool for writing file contents
pub struct WriteFileTool {
    config: Arc<FileToolsConfig>,
}

impl WriteFileTool {
    /// Creates a new write file tool with the given configuration.
    pub fn new(config: Arc<FileToolsConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ToolTrait for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        // Static fallback description
        "Write content to a file, creating a new file or completely replacing existing file content. Use to_llm_tool() for dynamic path-aware description."
    }

    fn to_llm_tool(&self) -> crate::traits::provider::Tool {
        let base_path = self
            .config
            .allowed_directories
            .first()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[no allowed directories configured]".to_string());

        let dynamic_description = format!(
            "Write content to a file, creating new or completely replacing existing file content.

Usage:
- File paths MUST be absolute paths starting with: {}
- Example: '{}/your_filename.txt'
- OVERWRITES entire file content - use edit_file for partial changes
- ALWAYS read file first if it exists to understand current content
- Creates parent directories automatically
- Requires user confirmation before execution

When to use:
- Creating new files
- Completely replacing file content
- When edit_file operations would be too complex

When NOT to use:
- Making small changes to existing files (use edit_file instead)
- When you haven't read the existing file first

Best practices:
- Provide complete, well-formatted content
- Consider impact on dependent files
- Include appropriate headers/imports as needed

Security: Restricted to application data directory, requires confirmation",
            base_path, base_path
        );

        crate::traits::provider::Tool {
            name: self.name().to_string(),
            tool_type: "function".to_string(),
            function: crate::traits::provider::ToolFunction {
                name: self.name().to_string(),
                description: dynamic_description,
                parameters: self.parameters_schema(),
            },
        }
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true // Always require confirmation for file writes
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let file_path = arguments.get("file_path")?.as_str()?;
        let new_content = arguments.get("content")?.as_str()?;

        // Try to validate path and read current content for diff
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let diff_preview = match path_validator.validate_path(file_path) {
            Ok(validated_path) => {
                if validated_path.exists() && validated_path.is_file() {
                    match fs::read_to_string(&validated_path) {
                        Ok(current_content) => {
                            let diff_display = generate_diff_display(&current_content, new_content);
                            let added_lines = new_content.lines().count();
                            let removed_lines = current_content.lines().count();
                            Some(DiffPreview {
                                added_lines,
                                removed_lines,
                                preview: diff_display,
                            })
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        // Prepare content preview (truncate if too long)
        let content_preview = if diff_preview.is_none() {
            if new_content.len() <= 1000 {
                Some(new_content.to_string())
            } else {
                Some(format!(
                    "{}...\n\n[Content truncated at 1000 characters for display]",
                    &new_content[..1000]
                ))
            }
        } else {
            None // Use diff preview instead
        };

        Some(ActionDescription {
            title: "Write File".to_string(),
            description: format!(
                "Write {} characters to file: {}",
                new_content.len(),
                file_path
            ),
            changes: ChangeDetail::FileWrite {
                file_path: file_path.to_string(),
                content_length: new_content.len(),
                diff_preview,
                content_preview,
            },
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' parameter")?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        // Validate the path (allowing directory creation for write operations)
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let validated_path = path_validator.validate_path_for_write(file_path)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = validated_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!(
                        "Failed to create parent directory '{}': {}. Check that you have the correct path and permissions. Use list_files to explore the directory structure.", 
                        parent.display(), e
                    ))?;
            }
        }

        // Write the file
        fs::write(&validated_path, content)
            .map_err(|e| format!(
                "Failed to write file '{}': {}. Check that the path is correct and you have write permissions.", 
                validated_path.display(), e
            ))?;

        // Return success message
        let result = json!({
            "status": "success",
            "action": "file_written",
            "details": {
                "file_path": validated_path.to_string_lossy(),
                "content_length": content.len()
            },
            "message": format!("File successfully written to: {}", validated_path.display())
        });

        debug!("Wrote file: {}", validated_path.display());
        Ok(serde_json::to_string_pretty(&result).unwrap())
    }
}

/// Tool for listing files in a directory
pub struct ListFilesTool {
    config: Arc<FileToolsConfig>,
}

impl ListFilesTool {
    /// Creates a new list files tool with the given configuration.
    pub fn new(config: Arc<FileToolsConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ToolTrait for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories in a specified path with optional pattern filtering.

Usage:
- Directory path must be absolute and within allowed application directories
- Returns detailed information about files and subdirectories
- Supports glob pattern filtering (e.g., '*.json', 'config.*')
- Non-recursive by default - shows immediate directory contents only
- Essential for exploring application file structure

When to use:
- Before read/edit operations to understand directory layout
- Finding specific files by pattern (configuration, templates, etc.)
- Exploring unknown directory structure
- Confirming file existence and getting exact names

When NOT to use:
- If you already know the exact file path (use read_file directly)
- For system directories (restricted to application data)

Output: Each entry includes name, size, modification time, and full path

Security: Restricted to application data directory, prevents directory traversal"
    }

    fn workflow_guidance(&self) -> Option<String> {
        Some("Use list_files first to discover directory structure and available files before other file operations".to_string())
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "directory_path": {
                    "type": "string",
                    "description": "Absolute path to the directory to list"
                },
                "pattern": {
                    "type": "string",
                    "description": "Optional glob pattern to filter files (e.g., '*.md', '*.txt')"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list files recursively (default: false)"
                }
            },
            "required": ["directory_path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let directory_path = arguments
            .get("directory_path")
            .or_else(|| arguments.get("path")) // Accept both parameter names for compatibility
            .and_then(|v| v.as_str())
            .ok_or("Missing 'directory_path' parameter")?;

        let pattern = arguments.get("pattern").and_then(|v| v.as_str());

        let recursive = arguments
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Validate the path
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let validated_path = path_validator.validate_path(directory_path)?;

        // Check that it's actually a directory
        if !validated_path.is_dir() {
            return Err(format!("Path is not a directory: {}", validated_path.display()).into());
        }

        // List files
        let mut files = Vec::new();

        if recursive {
            // Recursive listing using walkdir
            use std::path::Path;
            fn collect_files_recursive(
                dir: &Path,
                pattern: Option<&str>,
            ) -> Result<Vec<(PathBuf, std::fs::Metadata)>, Box<dyn Error + Send + Sync>>
            {
                let mut files = Vec::new();

                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        // Recurse into subdirectory
                        files.extend(collect_files_recursive(&path, pattern)?);
                    } else if path.is_file() {
                        // Check pattern if provided
                        if let Some(pattern) = pattern {
                            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                if glob_match(pattern, filename) {
                                    files.push((path, entry.metadata()?));
                                }
                            }
                        } else {
                            files.push((path, entry.metadata()?));
                        }
                    }
                }

                Ok(files)
            }

            files = collect_files_recursive(&validated_path, pattern)?;
        } else {
            // Non-recursive listing
            for entry in fs::read_dir(&validated_path)
                .map_err(|e| format!("Failed to read directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                if path.is_file() {
                    // Check pattern if provided
                    if let Some(pattern) = pattern {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if glob_match(pattern, filename) {
                                let metadata = entry
                                    .metadata()
                                    .map_err(|e| format!("Failed to read file metadata: {}", e))?;
                                files.push((path, metadata));
                            }
                        }
                    } else {
                        let metadata = entry
                            .metadata()
                            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
                        files.push((path, metadata));
                    }
                }
            }
        }

        // Sort files by name
        files.sort_by(|a, b| a.0.cmp(&b.0));

        if files.is_empty() {
            return Ok("No files found matching the criteria.".to_string());
        }

        // Format the file list
        let file_count = files.len();
        let mut output = format!(
            "Found {} file(s) in {}:\n\n",
            file_count,
            validated_path.display()
        );

        for (path, metadata) in &files {
            let relative_path = path.strip_prefix(&validated_path).unwrap_or(path);

            let size = metadata.len();
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| {
                    use std::time::SystemTime;
                    time.duration_since(SystemTime::UNIX_EPOCH).ok()
                })
                .map(|duration| {
                    use chrono::{DateTime, Utc};
                    let datetime = DateTime::<Utc>::from_timestamp(duration.as_secs() as i64, 0)
                        .unwrap_or_else(Utc::now);
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|| "Unknown".to_string());

            output.push_str(&format!(
                "- {} ({} bytes, modified: {})\n",
                relative_path.display(),
                size,
                modified
            ));
        }

        debug!(
            "Listed {} files in: {}",
            file_count,
            validated_path.display()
        );
        Ok(output)
    }
}

/// Tool for editing files using line-number based changes
pub struct EditFileTool {
    config: Arc<FileToolsConfig>,
}

impl EditFileTool {
    /// Creates a new edit file tool with the given configuration.
    pub fn new(config: Arc<FileToolsConfig>) -> Self {
        Self { config }
    }

    /// Parse edit instructions and convert them to line edits
    fn parse_edits(
        &self,
        content_lines: &[String],
        edits: &Value,
    ) -> Result<Vec<LineEdit>, Box<dyn Error + Send + Sync>> {
        let edits_array = edits.as_array().ok_or("Edits must be an array")?;

        let mut line_edits = Vec::new();

        for edit in edits_array {
            let operation_str = edit
                .get("operation")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'operation' field in edit")?;

            let operation = match operation_str {
                "replace" => EditOperation::Replace,
                "insert" => EditOperation::Insert,
                "delete" => EditOperation::Delete,
                _ => return Err(format!("Unknown operation: {}", operation_str).into()),
            };

            let start_line = edit
                .get("start_line")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'start_line' field in edit")? as usize;

            let end_line = edit
                .get("end_line")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(start_line);

            // Validate line numbers (1-indexed)
            if start_line == 0 || end_line == 0 {
                return Err("Line numbers must be 1-indexed (start from 1)".into());
            }

            if start_line > content_lines.len() + 1 {
                return Err(format!(
                    "Start line {} is beyond file length {}",
                    start_line,
                    content_lines.len()
                )
                .into());
            }

            if end_line > content_lines.len() + 1 {
                return Err(format!(
                    "End line {} is beyond file length {}",
                    end_line,
                    content_lines.len()
                )
                .into());
            }

            if start_line > end_line {
                return Err(format!(
                    "Start line {} cannot be greater than end line {}",
                    start_line, end_line
                )
                .into());
            }

            // Get old content (0-indexed for array access)
            let old_content: Vec<String> = match operation {
                EditOperation::Insert => Vec::new(),
                _ => {
                    let start_idx = (start_line - 1).min(content_lines.len());
                    let end_idx = end_line.min(content_lines.len());
                    content_lines[start_idx..end_idx].to_vec()
                }
            };

            // Get new content
            let new_content = edit
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| {
                    s.lines()
                        .map(|line| line.to_string())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_else(Vec::new);

            // Get context lines for preview (2-3 lines each side)
            let context_line_count = 2; // Could be made configurable in the future

            // Context before: lines preceding the edit
            let context_before_start = start_line.saturating_sub(1 + context_line_count);
            let context_before_end = start_line.saturating_sub(1);
            let context_before = (context_before_start..context_before_end)
                .filter_map(|i| content_lines.get(i).cloned())
                .collect::<Vec<String>>();

            // Context after: lines following the edit
            let context_after_start = end_line;
            let context_after_end = end_line + context_line_count;
            let context_after = (context_after_start..context_after_end)
                .filter_map(|i| content_lines.get(i).cloned())
                .collect::<Vec<String>>();

            line_edits.push(LineEdit {
                operation,
                start_line,
                end_line,
                old_content,
                new_content,
                context_before,
                context_after,
            });
        }

        // Sort edits by line number (reverse order for proper application)
        line_edits.sort_by(|a, b| b.start_line.cmp(&a.start_line));

        Ok(line_edits)
    }

    /// Apply line edits to content and return the new content
    fn apply_edits(
        &self,
        content_lines: &mut Vec<String>,
        line_edits: &[LineEdit],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        for edit in line_edits {
            match edit.operation {
                EditOperation::Replace => {
                    // Remove old lines and insert new ones
                    let start_idx = (edit.start_line - 1).min(content_lines.len());
                    let end_idx = edit.end_line.min(content_lines.len());

                    // Remove old lines
                    for _ in start_idx..end_idx {
                        if start_idx < content_lines.len() {
                            content_lines.remove(start_idx);
                        }
                    }

                    // Insert new lines
                    for (i, line) in edit.new_content.iter().enumerate() {
                        content_lines.insert(start_idx + i, line.clone());
                    }
                }
                EditOperation::Insert => {
                    // Insert new lines at the specified position
                    let insert_idx = (edit.start_line - 1).min(content_lines.len());
                    for (i, line) in edit.new_content.iter().enumerate() {
                        content_lines.insert(insert_idx + i, line.clone());
                    }
                }
                EditOperation::Delete => {
                    // Remove lines
                    let start_idx = (edit.start_line - 1).min(content_lines.len());
                    let end_idx = edit.end_line.min(content_lines.len());

                    for _ in start_idx..end_idx {
                        if start_idx < content_lines.len() {
                            content_lines.remove(start_idx);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if read_file or write_file was called recently for the same file
    fn check_recent_read(
        &self,
        file_path: &str,
        recent_calls: &Arc<Mutex<VecDeque<ToolCall>>>,
    ) -> bool {
        let calls = recent_calls.lock().unwrap();

        // Look for a recent read_file or write_file call for the same file
        // write_file is acceptable because it means we just created/modified the file content
        calls.iter().rev().take(5).any(|call| {
            (call.name == "read_file" || call.name == "write_file")
                && call
                    .file_path
                    .as_ref()
                    .map(|p| p == file_path)
                    .unwrap_or(false)
        })
    }
}

#[async_trait]
impl ToolTrait for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        // Static fallback description
        "Edit a file using precise line-number based operations for safe, incremental changes. Use to_llm_tool() for dynamic path-aware description."
    }

    fn workflow_guidance(&self) -> Option<String> {
        Some("MANDATORY: Call read_file first to get current content with line numbers before using edit_file".to_string())
    }

    fn to_llm_tool(&self) -> crate::traits::provider::Tool {
        let base_path = self
            .config
            .allowed_directories
            .first()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[no allowed directories configured]".to_string());

        let dynamic_description = format!(
            "Edit a file using precise line-number based operations for safe, incremental changes.

Usage:
- File paths MUST be absolute paths starting with: {}
- Example: '{}/your_filename.txt'
- MANDATORY: Call read_file first to get current content with line numbers
- Supports: replace, insert, delete operations
- Line numbers must match read_file output exactly (1-indexed)
- All edits atomic - succeed together or fail together
- Requires user confirmation before execution

Edit Operations:
1. REPLACE: Change lines start_line to end_line
   - new_content replaces everything between (inclusive)
   - Can replace single line (start_line = end_line) or multiple lines

2. INSERT: Add content before start_line  
   - new_content inserted as new lines
   - Original content shifts down

3. DELETE: Remove lines start_line to end_line
   - All lines between (inclusive) are removed
   - new_content should be empty

When to use:
- Preferred over write_file for existing files
- Making incremental changes while preserving structure
- When you need precise line-based control

When NOT to use:
- Creating new files (use write_file)
- Without reading file first (will be guided to read_file)

Security: Restricted to application data directory, atomic operations",
            base_path, base_path
        );

        crate::traits::provider::Tool {
            name: self.name().to_string(),
            tool_type: "function".to_string(),
            function: crate::traits::provider::ToolFunction {
                name: self.name().to_string(),
                description: dynamic_description,
                parameters: self.parameters_schema(),
            },
        }
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit"
                },
                "edits": {
                    "type": "array",
                    "description": "Array of edit operations to apply",
                    "items": {
                        "type": "object",
                        "properties": {
                            "operation": {
                                "type": "string",
                                "enum": ["replace", "insert", "delete"],
                                "description": "Type of edit operation"
                            },
                            "start_line": {
                                "type": "number",
                                "description": "Starting line number (1-indexed, from read_file output)"
                            },
                            "end_line": {
                                "type": "number",
                                "description": "Ending line number (1-indexed, inclusive). Optional for insert operations."
                            },
                            "content": {
                                "type": "string",
                                "description": "New content to insert or replace with. Not used for delete operations."
                            }
                        },
                        "required": ["operation", "start_line"]
                    }
                }
            },
            "required": ["file_path", "edits"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        // Always require confirmation for actual file edits
        // Guidance messages (when read_file wasn't called) won't reach the confirmation stage
        // because execute_with_context will return guidance before triggering confirmation
        true
    }

    fn describe_action(&self, arguments: &Value) -> Option<ActionDescription> {
        let file_path = match arguments.get("file_path").and_then(|v| v.as_str()) {
            Some(path) => path,
            None => {
                return Some(ActionDescription {
                    title: "Edit File".to_string(),
                    description: "Missing file_path parameter".to_string(),
                    changes: ChangeDetail::Generic {
                        items: vec!["Invalid parameters".to_string()],
                    },
                });
            }
        };

        let edits_value = match arguments.get("edits") {
            Some(edits) => edits,
            None => {
                return Some(ActionDescription {
                    title: "Edit File".to_string(),
                    description: "Missing edits parameter".to_string(),
                    changes: ChangeDetail::Generic {
                        items: vec!["Invalid parameters".to_string()],
                    },
                });
            }
        };

        // Try to read current content for preview
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let validated_path = match path_validator.validate_path(file_path) {
            Ok(path) => path,
            Err(err) => {
                return Some(ActionDescription {
                    title: "Edit File".to_string(),
                    description: format!("Invalid file path: {}", err),
                    changes: ChangeDetail::Generic {
                        items: vec![format!("Path validation error: {}", err)],
                    },
                });
            }
        };

        if !validated_path.is_file() {
            return Some(ActionDescription {
                title: "Edit File".to_string(),
                description: format!("File does not exist: {}", file_path),
                changes: ChangeDetail::Generic {
                    items: vec!["File not found".to_string()],
                },
            });
        }

        let current_content = match fs::read_to_string(&validated_path) {
            Ok(content) => content,
            Err(_err) => {
                return Some(ActionDescription {
                    title: "Edit File (Read Required)".to_string(),
                    description: format!("This edit requires reading the file first. Please call read_file('{}') to get current content with line numbers, then retry this edit.", file_path),
                    changes: ChangeDetail::Generic {
                        items: vec![
                            format!("Action needed: Call read_file('{}') first", file_path),
                            "This edit operation requires current file content for safety".to_string(),
                        ],
                    },
                });
            }
        };

        let content_lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();

        // Parse the edits to generate preview
        match self.parse_edits(&content_lines, edits_value) {
            Ok(line_edits) => {
                let total_lines_affected = line_edits
                    .iter()
                    .map(|edit| (edit.end_line - edit.start_line + 1).max(edit.new_content.len()))
                    .sum();

                Some(ActionDescription {
                    title: "Edit File".to_string(),
                    description: format!(
                        "Apply {} edit operation(s) to file: {}",
                        line_edits.len(),
                        file_path
                    ),
                    changes: ChangeDetail::FileEdit {
                        file_path: file_path.to_string(),
                        edits: line_edits,
                        total_lines_affected,
                        total_lines_in_file: content_lines.len(),
                    },
                })
            }
            Err(err) => Some(ActionDescription {
                title: "Edit File".to_string(),
                description: format!("Invalid edit operations for file: {}", file_path),
                changes: ChangeDetail::Generic {
                    items: vec![format!("Edit parsing error: {}", err)],
                },
            }),
        }
    }

    async fn execute_with_context(
        &self,
        arguments: Value,
        recent_calls: Arc<Mutex<VecDeque<ToolCall>>>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' parameter")?;

        let edits_value = arguments.get("edits").ok_or("Missing 'edits' parameter")?;

        // Check if read_file was called recently for this file
        if !self.check_recent_read(file_path, &recent_calls) {
            // Instead of erroring, provide helpful guidance to the LLM
            let guidance_message = json!({
                "status": "guidance_needed",
                "action": "read_file_required",
                "message": format!(
                    "To edit this file, I need to read its current content with line numbers first. Please call read_file('{}') and then retry this edit operation.",
                    file_path
                ),
                "suggested_next_steps": [
                    format!("1. Call read_file with file_path: '{}'", file_path),
                    "2. Review the current content and line numbers",
                    "3. Retry this edit_file operation with the same parameters"
                ],
                "details": {
                    "file_path": file_path,
                    "reason": "Line-number based editing requires current file content for safety and accuracy"
                }
            });

            debug!(
                "EditFileTool: Guiding LLM to read file first: {}",
                file_path
            );
            return Ok(serde_json::to_string_pretty(&guidance_message).unwrap());
        }

        // Validate the path
        let path_validator = PathValidator::new(self.config.allowed_directories.clone());
        let validated_path = path_validator.validate_path(file_path)?;

        // Check that it's actually a file
        if !validated_path.is_file() {
            return Err(format!("Path is not a file: {}", validated_path.display()).into());
        }

        // Read current content
        let current_content = fs::read_to_string(&validated_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let mut content_lines: Vec<String> =
            current_content.lines().map(|s| s.to_string()).collect();

        // Parse and apply edits
        let line_edits = self.parse_edits(&content_lines, edits_value)?;
        self.apply_edits(&mut content_lines, &line_edits)?;

        // Write the modified content back to the file
        let new_content = content_lines.join("\n");
        fs::write(&validated_path, &new_content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        debug!("Edited file: {}", validated_path.display());

        // Return success message
        let result = json!({
            "status": "success",
            "action": "file_edited",
            "details": {
                "file_path": validated_path.to_string_lossy(),
                "edits_applied": line_edits.len(),
                "final_line_count": content_lines.len()
            },
            "message": format!("Successfully applied {} edit(s) to file: {}", line_edits.len(), validated_path.display())
        });

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        // This fallback version creates an empty call stack for contexts where it's not available
        let empty_calls = Arc::new(Mutex::new(VecDeque::new()));
        self.execute_with_context(arguments, empty_calls).await
    }
}

/// Simple glob pattern matching (supports * and ? wildcards)
fn glob_match(pattern: &str, text: &str) -> bool {
    // Convert glob pattern to regex
    let regex_pattern = pattern
        .replace(".", "\\.")
        .replace("*", ".*")
        .replace("?", ".");

    if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        regex.is_match(text)
    } else {
        // Fallback to simple contains check if regex compilation fails
        text.contains(&pattern.replace("*", "").replace("?", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_validator_allows_valid_paths() {
        let temp_dir = TempDir::new().unwrap();
        let validator = PathValidator::new(vec![temp_dir.path().to_path_buf()]);

        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let result = validator.validate_path(test_file.to_str().unwrap());
        if let Err(e) = &result {
            eprintln!("Validation failed for path: {:?}", test_file);
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_validator_rejects_forbidden_paths() {
        let temp_dir = TempDir::new().unwrap();
        let validator = PathValidator::new(vec![temp_dir.path().to_path_buf()]);

        let result = validator.validate_path("/etc/passwd");
        assert!(result.is_err());

        let result = validator.validate_path("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_diff_display_simple_change() {
        let current = "# Test File\n\nOriginal content.\nStay the same.";
        let new_content = "# Test File\n\nNew content.\nStay the same.";

        let result = generate_diff_display(current, new_content);

        assert!(result.contains("```diff"));
        assert!(result.contains("- Original content."));
        assert!(result.contains("+ New content."));
        assert!(result.contains("  Stay the same."));
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.txt", "file.txt"));
        assert!(glob_match("test.*", "test.md"));
        assert!(glob_match("file?.txt", "file1.txt"));
        assert!(!glob_match("*.txt", "file.md"));
    }
}

use serde::{Deserialize, Serialize};

/// Categories of evaluation tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Tool calling tasks - objective, can measure correctness
    ToolCalling,
    /// Content generation tasks - subjective, rate 1-5
    Generation,
    /// Reasoning and planning tasks - subjective evaluation
    Reasoning,
    /// Edge cases - testing model behavior boundaries
    EdgeCases,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::ToolCalling => "tool_calling",
            Category::Generation => "generation",
            Category::Reasoning => "reasoning",
            Category::EdgeCases => "edge_cases",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::ToolCalling => "Tool Calling",
            Category::Generation => "Content Generation",
            Category::Reasoning => "Reasoning & Planning",
            Category::EdgeCases => "Edge Cases",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A single evaluation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalTask {
    /// Unique task identifier
    pub id: String,
    /// Task category
    pub category: Category,
    /// The prompt to send to the model
    pub prompt: String,
    /// Expected tools to be called (for tool_calling category)
    #[serde(default)]
    pub expected_tools: Vec<String>,
    /// Evaluation criteria for subjective tasks
    #[serde(default)]
    pub evaluation_criteria: Vec<String>,
    /// Description of what the task tests
    #[serde(default)]
    pub description: String,
    /// Whether this task should NOT trigger tool calls
    #[serde(default)]
    pub expect_no_tools: bool,
}

/// Result of a tool call made by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Tool name that was called
    pub name: String,
    /// Arguments passed to the tool
    pub arguments: serde_json::Value,
    /// Whether the tool call was valid/expected
    pub was_expected: bool,
}

/// Result of running a single evaluation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    /// Task that was evaluated
    pub task_id: String,
    /// Category of the task
    pub category: Category,
    /// Model that was tested
    pub model: String,
    /// Provider (ollama, groq, etc.)
    pub provider: String,
    /// The prompt that was sent
    pub prompt: String,
    /// Response content from the model
    pub response: String,
    /// Tools that were called
    pub tools_called: Vec<ToolCallResult>,
    /// Tool calling accuracy (0.0 - 1.0) for objective tasks
    pub tool_accuracy: Option<f32>,
    /// Quality score (1-5) for subjective tasks (set manually or by reviewer)
    pub quality_score: Option<u8>,
    /// Time taken for response in milliseconds
    pub response_time_ms: u64,
    /// Token usage if available
    pub tokens_used: Option<TokenUsage>,
    /// Whether the task was successful overall
    pub success: bool,
    /// Error message if task failed
    pub error: Option<String>,
    /// Timestamp when evaluation was run
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Collection of tasks loaded from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSet {
    /// Name of this task set
    pub name: String,
    /// Description
    pub description: String,
    /// The tasks in this set
    pub tasks: Vec<EvalTask>,
}

impl TaskSet {
    /// Load a task set from a JSON file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let task_set: TaskSet = serde_json::from_str(&content)?;
        Ok(task_set)
    }

    /// Load all task sets from a directory
    pub fn load_all(dir: &std::path::Path) -> anyhow::Result<Vec<Self>> {
        let mut task_sets = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                task_sets.push(Self::from_file(&path)?);
            }
        }
        Ok(task_sets)
    }
}

/// Configuration for models to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalConfig {
    /// Models to evaluate
    pub models: Vec<ModelSpec>,
}

/// Specification for a model to test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Provider name (ollama, groq)
    pub provider: String,
    /// Model identifier
    pub model: String,
    /// Optional display name
    pub name: Option<String>,
    /// Provider-specific configuration
    #[serde(default)]
    pub config: std::collections::HashMap<String, String>,
}

impl EvalConfig {
    /// Load config from a JSON file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: EvalConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

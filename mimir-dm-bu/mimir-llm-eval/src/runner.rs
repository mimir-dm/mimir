use std::collections::HashMap;
use std::time::Instant;

use anyhow::{Context, Result};
use mimir_dm_llm::config::{EndpointType, ModelConfig};
use mimir_dm_llm::providers::groq::GroqProvider;
use mimir_dm_llm::providers::ollama::OllamaProvider;
use mimir_dm_llm::traits::{LlmProvider, Message, Tool, ToolFunction};

use crate::tasks::{Category, EvalResult, EvalTask, ModelSpec, TokenUsage, ToolCallResult};

/// Progress callback for single-model evaluation: (current, total, task_id)
type ProgressCallback<'a> = Option<&'a dyn Fn(usize, usize, &str)>;

/// Progress callback for multi-model comparison: (model_name, current, total, task_id)
type ComparisonProgressCallback<'a> = Option<&'a dyn Fn(&str, usize, usize, &str)>;

/// Wrapper enum for providers since LlmProvider isn't dyn-compatible
enum ProviderWrapper {
    Ollama(OllamaProvider),
    Groq(GroqProvider),
}

impl ProviderWrapper {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> std::result::Result<mimir_dm_llm::ChatResponse, mimir_dm_llm::LlmError> {
        match self {
            ProviderWrapper::Ollama(p) => {
                p.chat(messages, tools, None, None, None, None, None, None)
                    .await
            }
            ProviderWrapper::Groq(p) => {
                p.chat(messages, tools, None, None, None, None, None, None)
                    .await
            }
        }
    }
}

/// Evaluation runner that executes tasks against models
pub struct EvalRunner {
    /// System prompt to use for all evaluations
    system_prompt: String,
    /// Available tools for the model to call
    tools: Vec<Tool>,
}

impl EvalRunner {
    /// Create a new evaluation runner
    pub fn new() -> Self {
        Self {
            system_prompt: Self::default_system_prompt(),
            tools: Self::default_tools(),
        }
    }

    /// Create runner with custom system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    /// Create runner with custom tools
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    fn default_system_prompt() -> String {
        r#"You are a helpful D&D (Dungeons & Dragons 5th Edition) game master assistant.
You help dungeon masters run their games by providing information about spells, monsters,
characters, and game rules. You can also help create NPCs, encounters, and descriptions.

When asked about specific game content (spells, monsters, characters), use the available
tools to look up accurate information rather than relying on memory."#
            .to_string()
    }

    fn default_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "search_spells".to_string(),
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "search_spells".to_string(),
                    description: "Search for D&D spells by name or criteria".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query for spell name"
                            },
                            "level": {
                                "type": "integer",
                                "description": "Filter by spell level (0-9)"
                            },
                            "school": {
                                "type": "string",
                                "description": "Filter by magic school"
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
            Tool {
                name: "get_monster_details".to_string(),
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "get_monster_details".to_string(),
                    description: "Get detailed stats for a D&D monster".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Name of the monster"
                            }
                        },
                        "required": ["name"]
                    }),
                },
            },
            Tool {
                name: "list_players".to_string(),
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "list_players".to_string(),
                    description: "List all players in the current campaign".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                },
            },
            Tool {
                name: "get_character_details".to_string(),
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "get_character_details".to_string(),
                    description: "Get details about a player character".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "character_name": {
                                "type": "string",
                                "description": "Name of the character"
                            }
                        },
                        "required": ["character_name"]
                    }),
                },
            },
        ]
    }

    /// Create a provider from a model specification
    fn create_provider(&self, spec: &ModelSpec) -> Result<ProviderWrapper> {
        let mut config_map = spec.config.clone();

        match spec.provider.as_str() {
            "ollama" => {
                config_map
                    .entry("base_url".to_string())
                    .or_insert_with(|| "http://localhost:11434".to_string());

                let config = ModelConfig {
                    name: spec.name.clone().unwrap_or_else(|| spec.model.clone()),
                    supported_endpoints: vec![EndpointType::Chat],
                    provider: "ollama".to_string(),
                    model: spec.model.clone(),
                    config: Some(config_map),
                    limit: None,
                };

                let provider = OllamaProvider::new(config)
                    .context("Failed to create Ollama provider")?;
                Ok(ProviderWrapper::Ollama(provider))
            }
            "groq" => {
                if !config_map.contains_key("api_key") {
                    if let Ok(key) = std::env::var("GROQ_API_KEY") {
                        config_map.insert("api_key".to_string(), key);
                    } else {
                        anyhow::bail!("Groq requires api_key in config or GROQ_API_KEY env var");
                    }
                }

                let config = ModelConfig {
                    name: spec.name.clone().unwrap_or_else(|| spec.model.clone()),
                    supported_endpoints: vec![EndpointType::Chat],
                    provider: "groq".to_string(),
                    model: spec.model.clone(),
                    config: Some(config_map),
                    limit: None,
                };

                let provider = GroqProvider::new(config)
                    .context("Failed to create Groq provider")?;
                Ok(ProviderWrapper::Groq(provider))
            }
            other => anyhow::bail!("Unknown provider: {}", other),
        }
    }

    /// Run a single evaluation task against a model
    pub async fn run_task(&self, task: &EvalTask, spec: &ModelSpec) -> EvalResult {
        let start = Instant::now();
        let timestamp = chrono::Utc::now();

        let provider = match self.create_provider(spec) {
            Ok(p) => p,
            Err(e) => {
                return EvalResult {
                    task_id: task.id.clone(),
                    category: task.category,
                    model: spec.model.clone(),
                    provider: spec.provider.clone(),
                    prompt: task.prompt.clone(),
                    response: String::new(),
                    tools_called: vec![],
                    tool_accuracy: None,
                    quality_score: None,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    tokens_used: None,
                    success: false,
                    error: Some(format!("Failed to create provider: {}", e)),
                    timestamp,
                };
            }
        };

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content: task.prompt.clone(),
                tool_call_id: None,
            },
        ];

        // Only provide tools for tool calling tasks, or to test if model correctly avoids them
        let tools = if task.category == Category::ToolCalling || task.expect_no_tools {
            Some(self.tools.clone())
        } else {
            None
        };

        let response = provider.chat(messages, tools).await;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match response {
            Ok(chat_response) => {
                let tools_called: Vec<ToolCallResult> = chat_response
                    .tool_calls
                    .unwrap_or_default()
                    .into_iter()
                    .map(|tc| {
                        let was_expected = task.expected_tools.contains(&tc.function.name);
                        // Arguments is already a serde_json::Value in some cases, or a String
                        let arguments = tc.function.arguments.clone();
                        ToolCallResult {
                            name: tc.function.name,
                            arguments,
                            was_expected,
                        }
                    })
                    .collect();

                // Calculate tool accuracy for tool calling tasks
                let tool_accuracy = if task.category == Category::ToolCalling {
                    Some(self.calculate_tool_accuracy(&task.expected_tools, &tools_called))
                } else if task.expect_no_tools {
                    // For expect_no_tools, accuracy is 1.0 if no tools called, 0.0 otherwise
                    Some(if tools_called.is_empty() { 1.0 } else { 0.0 })
                } else {
                    None
                };

                let tokens_used = chat_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                });

                let success = match task.category {
                    Category::ToolCalling => tool_accuracy.unwrap_or(0.0) > 0.5,
                    _ => !chat_response.content.is_empty(),
                };

                EvalResult {
                    task_id: task.id.clone(),
                    category: task.category,
                    model: spec.model.clone(),
                    provider: spec.provider.clone(),
                    prompt: task.prompt.clone(),
                    response: chat_response.content,
                    tools_called,
                    tool_accuracy,
                    quality_score: None, // Set manually for subjective tasks
                    response_time_ms: elapsed_ms,
                    tokens_used,
                    success,
                    error: None,
                    timestamp,
                }
            }
            Err(e) => EvalResult {
                task_id: task.id.clone(),
                category: task.category,
                model: spec.model.clone(),
                provider: spec.provider.clone(),
                prompt: task.prompt.clone(),
                response: String::new(),
                tools_called: vec![],
                tool_accuracy: None,
                quality_score: None,
                response_time_ms: elapsed_ms,
                tokens_used: None,
                success: false,
                error: Some(format!("LLM error: {}", e)),
                timestamp,
            },
        }
    }

    /// Calculate tool calling accuracy
    fn calculate_tool_accuracy(&self, expected: &[String], actual: &[ToolCallResult]) -> f32 {
        if expected.is_empty() && actual.is_empty() {
            return 1.0;
        }
        if expected.is_empty() {
            return 0.0; // Called tools when none expected
        }

        let actual_names: Vec<&str> = actual.iter().map(|t| t.name.as_str()).collect();

        let mut correct = 0;
        for exp in expected {
            if actual_names.contains(&exp.as_str()) {
                correct += 1;
            }
        }

        // Penalize for extra unexpected calls
        let unexpected = actual.iter().filter(|t| !t.was_expected).count();
        let total = expected.len() + unexpected;

        if total == 0 {
            1.0
        } else {
            correct as f32 / total as f32
        }
    }

    /// Run all tasks against a single model
    pub async fn run_all_tasks(
        &self,
        tasks: &[EvalTask],
        spec: &ModelSpec,
        progress_callback: ProgressCallback<'_>,
    ) -> Vec<EvalResult> {
        let mut results = Vec::new();
        let total = tasks.len();

        for (i, task) in tasks.iter().enumerate() {
            if let Some(callback) = progress_callback {
                callback(i + 1, total, &task.id);
            }

            let result = self.run_task(task, spec).await;
            results.push(result);
        }

        results
    }

    /// Run all tasks against multiple models
    pub async fn run_comparison(
        &self,
        tasks: &[EvalTask],
        models: &[ModelSpec],
        progress_callback: ComparisonProgressCallback<'_>,
    ) -> HashMap<String, Vec<EvalResult>> {
        let mut all_results = HashMap::new();

        for spec in models {
            let model_key = format!("{}:{}", spec.provider, spec.model);

            let mut results = Vec::new();
            let total = tasks.len();

            for (i, task) in tasks.iter().enumerate() {
                if let Some(callback) = progress_callback {
                    callback(&model_key, i + 1, total, &task.id);
                }

                let result = self.run_task(task, spec).await;
                results.push(result);
            }

            all_results.insert(model_key, results);
        }

        all_results
    }
}

impl Default for EvalRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_accuracy_all_correct() {
        let runner = EvalRunner::new();
        let expected = vec!["search_spells".to_string()];
        let actual = vec![ToolCallResult {
            name: "search_spells".to_string(),
            arguments: serde_json::json!({}),
            was_expected: true,
        }];
        assert_eq!(runner.calculate_tool_accuracy(&expected, &actual), 1.0);
    }

    #[test]
    fn test_tool_accuracy_none_expected_none_called() {
        let runner = EvalRunner::new();
        let expected: Vec<String> = vec![];
        let actual: Vec<ToolCallResult> = vec![];
        assert_eq!(runner.calculate_tool_accuracy(&expected, &actual), 1.0);
    }

    #[test]
    fn test_tool_accuracy_with_unexpected() {
        let runner = EvalRunner::new();
        let expected = vec!["search_spells".to_string()];
        let actual = vec![
            ToolCallResult {
                name: "search_spells".to_string(),
                arguments: serde_json::json!({}),
                was_expected: true,
            },
            ToolCallResult {
                name: "list_players".to_string(),
                arguments: serde_json::json!({}),
                was_expected: false,
            },
        ];
        // 1 correct out of 2 total (1 expected + 1 unexpected)
        assert_eq!(runner.calculate_tool_accuracy(&expected, &actual), 0.5);
    }
}

//! Traits for LLM providers and related functionality

pub mod context;
pub mod provider;
pub mod tool;

// Re-export commonly used types
pub use provider::{
    ChatResponse, CompletionResponse, EmbeddingResponse, LlmError, LlmProvider, Message, ModelInfo,
    ModelPullProgress, RateLimitState, Timing, Tool, ToolCall, ToolCallFunction, ToolFunction,
    Usage,
};

pub use context::ToolContext;
pub use tool::{
    ActionDescription, ChangeDetail, EditOperation, LineEdit, RiskLevel, Tool as ToolTrait,
    ToolCall as ToolCallContext,
};

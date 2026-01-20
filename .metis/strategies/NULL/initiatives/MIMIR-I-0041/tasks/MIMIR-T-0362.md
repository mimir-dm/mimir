---
id: v0-5-llm-chat-assistant-design
level: task
title: "v0.5 LLM Chat Assistant Design"
short_code: "MIMIR-T-0362"
created_at: 2026-01-20T01:11:23.782373+00:00
updated_at: 2026-01-20T01:11:23.782373+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 LLM Chat Assistant Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Design the in-app LLM chat assistant that provides campaign authoring assistance by consuming the same MCP tools that Claude Code uses. One tool interface, two clients (CLI and UI).

## Acceptance Criteria
- [ ] Chat UI component design
- [ ] MCP tool consumption architecture
- [ ] LLM provider abstraction (local inference + API options)
- [ ] Conversation/context management
- [ ] Tool result rendering in chat

## Architecture

### Key Principle: MCP Tools as Single Interface

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Tool Layer                          │
│  (list_campaigns, create_module, search_monsters, etc.)     │
└─────────────────────┬───────────────────┬───────────────────┘
                      │                   │
         ┌────────────┴────┐    ┌─────────┴─────────┐
         │  Claude Code    │    │   Mimir UI Chat   │
         │  (CLI Client)   │    │   (GUI Client)    │
         └─────────────────┘    └───────────────────┘
```

Both clients use the exact same tools. The UI chat is just a graphical wrapper around MCP tool invocation.

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Chat Panel (Vue)                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Message List                                        │    │
│  │  - User messages                                     │    │
│  │  - Assistant messages                                │    │
│  │  - Tool invocation cards (collapsible)               │    │
│  │  - Tool results (formatted)                          │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Input Area                                          │    │
│  │  - Text input                                        │    │
│  │  - Send button                                       │    │
│  │  - Context indicator (active campaign/module)        │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│                    Chat Service (Rust)                       │
│  - Manages conversation state                                │
│  - Formats tool definitions for LLM                         │
│  - Executes tool calls via MCP interface                    │
│  - Streams responses to frontend                            │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│                   LLM Provider Abstraction                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Local LLM   │  │ Anthropic   │  │ OpenAI-compatible   │  │
│  │ (Ollama)    │  │ API         │  │ API                 │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Chat Service Design

### Rust Service Trait

```rust
#[async_trait]
pub trait ChatService: Send + Sync {
    /// Start a new conversation
    async fn new_conversation(&self) -> Result<ConversationId>;
    
    /// Send a message and stream the response
    async fn send_message(
        &self,
        conversation_id: ConversationId,
        message: &str,
    ) -> Result<impl Stream<Item = ChatEvent>>;
    
    /// Get conversation history
    async fn get_history(&self, conversation_id: ConversationId) -> Result<Vec<ChatMessage>>;
    
    /// Clear conversation
    async fn clear_conversation(&self, conversation_id: ConversationId) -> Result<()>;
}

#[derive(Debug)]
pub enum ChatEvent {
    /// Text chunk from LLM
    TextDelta(String),
    
    /// LLM is invoking a tool
    ToolCall { name: String, arguments: Value },
    
    /// Tool execution result
    ToolResult { name: String, result: Value },
    
    /// Response complete
    Done,
    
    /// Error occurred
    Error(String),
}

pub struct ChatMessage {
    pub role: Role,  // User, Assistant, Tool
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
    pub timestamp: DateTime<Utc>,
}
```

### Tool Execution Flow

```rust
impl ChatServiceImpl {
    async fn execute_tool_call(&self, call: &ToolCall) -> Result<Value> {
        // Route to appropriate MCP tool
        match call.name.as_str() {
            "list_campaigns" => {
                let args: ListCampaignsArgs = serde_json::from_value(call.arguments)?;
                let result = self.campaign_service.list(args.include_archived).await?;
                Ok(serde_json::to_value(result)?)
            }
            "create_module" => {
                let args: CreateModuleArgs = serde_json::from_value(call.arguments)?;
                let result = self.module_service.create(args.into()).await?;
                Ok(serde_json::to_value(result)?)
            }
            "search_monsters" => {
                let args: SearchMonstersArgs = serde_json::from_value(call.arguments)?;
                let result = self.catalog_service.search_monsters(args.into()).await?;
                Ok(serde_json::to_value(result)?)
            }
            // ... all other MCP tools
            _ => Err(ServiceError::UnknownTool(call.name.clone())),
        }
    }
}
```

## LLM Provider Abstraction

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a response with tool use capability
    async fn generate(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
        system_prompt: &str,
    ) -> Result<impl Stream<Item = LlmEvent>>;
    
    /// Check if provider is available
    async fn health_check(&self) -> Result<bool>;
}

pub enum LlmEvent {
    TextDelta(String),
    ToolCall { id: String, name: String, arguments: Value },
    Done,
    Error(String),
}
```

### Provider Implementations

#### Local (Ollama)
```rust
pub struct OllamaProvider {
    base_url: String,
    model: String,  // e.g., "llama3.1:8b", "qwen2.5:14b"
}
```

#### Anthropic API
```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,  // e.g., "claude-sonnet-4-20250514"
}
```

#### OpenAI-compatible
```rust
pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: Option<String>,
    model: String,
}
```

## System Prompt

The chat assistant gets a system prompt that:
1. Describes Mimir and its purpose
2. Lists available tools with descriptions
3. Provides context about the active campaign/module
4. Sets tone and behavior expectations

```rust
fn build_system_prompt(context: &ChatContext) -> String {
    format!(r#"
You are a D&D campaign authoring assistant within Mimir, a dungeon master toolkit.

## Current Context
- Active Campaign: {campaign}
- Active Module: {module}

## Available Tools
You have access to tools for managing campaigns, modules, characters, 
documents, and searching the D&D 5e catalog. Use these tools to help 
the DM create and organize their campaign content.

## Behavior
- Be helpful and knowledgeable about D&D 5e
- Use tools to take actions, don't just describe what you could do
- When creating content, ask clarifying questions if needed
- Format responses for readability (markdown is supported)
"#,
        campaign = context.campaign_name.as_deref().unwrap_or("None"),
        module = context.module_name.as_deref().unwrap_or("None"),
    )
}
```

## Frontend Components

### Pinia Store

```typescript
interface ChatState {
  messages: ChatMessage[];
  isStreaming: boolean;
  error: string | null;
  conversationId: string | null;
}

export const useChatStore = defineStore('chat', {
  state: (): ChatState => ({
    messages: [],
    isStreaming: false,
    error: null,
    conversationId: null,
  }),

  actions: {
    async sendMessage(content: string) {
      this.messages.push({ role: 'user', content, timestamp: new Date() });
      this.isStreaming = true;
      
      try {
        // Stream response from backend
        const stream = await invoke('chat_send_message', {
          conversationId: this.conversationId,
          message: content,
        });
        
        let assistantMessage = { role: 'assistant', content: '', toolCalls: [] };
        this.messages.push(assistantMessage);
        
        for await (const event of stream) {
          switch (event.type) {
            case 'text_delta':
              assistantMessage.content += event.text;
              break;
            case 'tool_call':
              assistantMessage.toolCalls.push(event);
              break;
            case 'tool_result':
              // Update the tool call with its result
              break;
          }
        }
      } finally {
        this.isStreaming = false;
      }
    },
  },
});
```

### Vue Components

```
ChatPanel.vue
├── ChatMessageList.vue
│   ├── UserMessage.vue
│   ├── AssistantMessage.vue
│   └── ToolCallCard.vue (collapsible, shows tool name + result)
├── ChatInput.vue
└── ChatContextBar.vue (shows active campaign/module)
```

## Tool Result Rendering

Tool results should render nicely in the chat:

| Tool | Rendering |
|------|-----------|
| `search_monsters` | Monster cards with name, CR, type |
| `list_characters` | Character list with icons for PC/NPC |
| `get_module_details` | Module summary card |
| `create_*` | Success message with link to created item |
| Errors | Error alert with actionable message |

## Configuration

User settings for chat:

```typescript
interface ChatSettings {
  provider: 'local' | 'anthropic' | 'openai';
  
  // Local settings
  localModel: string;
  ollamaUrl: string;
  
  // API settings
  anthropicApiKey?: string;
  openaiApiKey?: string;
  openaiBaseUrl?: string;
  
  // Behavior
  streamResponses: boolean;
  showToolCalls: boolean;  // collapse or show tool invocations
}
```

## Dependencies

- Depends on: [[MIMIR-T-0359]] MCP Tool Specification (defines available tools)
- Depends on: [[MIMIR-T-0358]] Service Layer (chat service calls other services)

## Progress

*To be updated during implementation*
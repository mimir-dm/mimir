---
id: llm-mock-provider-for-offline
level: task
title: "LLM Mock Provider for Offline Testing"
short_code: "MIMIR-T-0336"
created_at: 2026-01-14T01:50:48.637364+00:00
updated_at: 2026-01-14T01:50:48.637364+00:00
parent: MIMIR-I-0039
blocked_by: [MIMIR-I-0037]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0039
---

# LLM Mock Provider for Offline Testing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Create a mock LLM provider in `mimir-dm-llm` that enables offline testing without requiring a running Ollama service.

## Scope

**Target: `crates/mimir-dm-llm/`**

Current state:
- Tests require live Ollama service (`require_ollama!` macro)
- `is_ollama_available()` check skips tests when service unavailable
- Tests in `tests/ollama.rs` and `tests/model_management.rs` are integration-only

**Goal:** Add a mock provider that returns predictable responses for unit testing.

## Acceptance Criteria

- [ ] Create `MockLlmProvider` implementing the provider trait
- [ ] Mock provider returns configurable/predictable responses
- [ ] Add unit tests that use mock provider (no Ollama required)
- [ ] Test error conditions (connection failure, timeout, invalid response)
- [ ] Test streaming response handling with mocks
- [ ] Existing Ollama integration tests continue to work
- [ ] Document mock usage in test utilities

## Implementation Notes

### Technical Approach

**Mock Provider Pattern:**
```rust
pub struct MockLlmProvider {
    responses: Vec<String>,
    should_fail: bool,
    delay_ms: Option<u64>,
}

impl MockLlmProvider {
    pub fn with_responses(responses: Vec<String>) -> Self { ... }
    pub fn failing() -> Self { ... }
}

impl LlmProvider for MockLlmProvider {
    async fn chat(&self, messages: &[Message]) -> Result<Response> { ... }
    async fn complete(&self, prompt: &str) -> Result<String> { ... }
}
```

**Test Scenarios:**
1. Successful chat completion
2. Streaming responses
3. Connection errors
4. Rate limiting behavior
5. Invalid model responses

### Files to Create/Modify
- `src/mock.rs` - New mock provider
- `src/lib.rs` - Export mock (behind `#[cfg(test)]` or feature flag)
- `tests/mock_provider.rs` - Unit tests using mock

### Risk Considerations
- Mock must accurately simulate real provider behavior
- Consider using a feature flag to control mock availability
- Streaming simulation requires careful handling

### Testing with angreal

Run LLM tests (will skip Ollama-dependent tests when unavailable):
```bash
# Run core tests including mimir-dm-llm
angreal test unit --core

# Verify mock provider tests run without Ollama
angreal test coverage --core
```

With mock provider, tests should pass in CI without requiring Ollama service.

## Status Updates **[REQUIRED]**

### 2026-01-14 - Blocked by Schema Hardening

This task is blocked by [[MIMIR-I-0037]] (Schema-First Catalog Model Hardening).

**Reason:** The LLM mock provider will need to return mock catalog data (monsters, spells, classes) for context generation. If we build mocks against the current `serde_json::Value` fields, they'll need to be rewritten once schema hardening replaces those with typed structs.

**Unblock condition:** Complete MIMIR-I-0037 or at minimum complete the monster/spell/class type migrations.
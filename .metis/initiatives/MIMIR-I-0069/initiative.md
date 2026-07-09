---
id: mcp-tool-registry-with-typed
level: initiative
title: "MCP tool registry with typed arguments"
short_code: "MIMIR-I-0069"
created_at: 2026-07-08T11:07:57.220182+00:00
updated_at: 2026-07-08T11:14:17.052638+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: mcp-tool-registry-with-typed
---

# MCP tool registry with typed arguments Initiative

## Context **[REQUIRED]**

Candidate #1 (Strong, top recommendation) from the 2026-07-07 architecture review.

The MCP tool seam is shallow four times over. Adding one tool touches four places
that must agree by hand:

1. Definition fn in `crates/mimir-mcp/src/tools/*.rs` (~25 lines of hand-built schema)
2. `get_tools()` vec in `crates/mimir-mcp/src/handler.rs`
3. `execute_tool()` match arm in `handler.rs`
4. `EXPECTED_TOOLS` test list in `handler.rs`

(Adding `update_module_monster` on 2026-07-07 touched exactly these four.)

Separately, every handler hand-parses `serde_json::Value`
(`args.get("x").and_then(|v| v.as_str())`), so the published JSON schema and the
actual parser are two independent copies of the same contract with nothing keeping
them in sync. The `ServiceError → McpError` mapping is also copy-pasted per handler
(written three times during the fc5a3d5 consolidation alone).

Scale: 55 tools; `tools/character.rs` alone is 1,129 lines, mostly arg parsing.
Deletion test passes: deleting `execute_tool()`, `get_tools()`, and
`EXPECTED_TOOLS` into a registry concentrates complexity in one module.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- One registration point per tool: typed argument struct + handler + metadata as a single unit
- JSON schema generated from the same struct that deserializes the arguments (drift becomes impossible)
- Dispatch, arg deserialization, and ServiceError→McpError normalization live once in the registry
- `EXPECTED_TOOLS` and the count/route bookkeeping tests dissolve (registry is the list)
- Handlers unit-testable with struct literals instead of JSON fixtures

**Non-Goals:**
- Changing any tool's published name, argument names, or response shape (wire-compatible refactor)
- Touching the Tauri command seam (separate initiatives)
- Replacing rust-mcp-sdk

## Detailed Design **[REQUIRED]**

Rough shape (to be refined at design phase):

- A `ToolDef` trait or struct: `{ name, description, ArgStruct: DeserializeOwned + JsonSchema-ish, handler: async fn(&McpContext, Args) -> Result<Value, McpError> }`
- Schema generation: either a small derive/macro over the arg struct or `schemars`-style generation feeding `ToolInputSchema`
- A registry (static slice or builder) that `MimirHandler` consumes for both `list_tools` and `call_tool`
- `impl From<ServiceError> for McpError` as the single error-mapping point in dispatch (absorbs MIMIR-I-0073)
- Migrate tools family-by-family (campaign → module → document → character → map → homebrew → mapgen → catalog); parity guaranteed by the existing functional tests in `handler.rs` and the stdio e2e test, which are wire-level and stay unchanged

## Alternatives Considered **[REQUIRED]**

- **Status quo + discipline**: rejected — the four-touch ceremony already produced drift and capability gaps under normal care.
- **Full proc-macro DSL**: deferred — a declarative macro or plain data table gets 90% of the leverage without proc-macro build cost; revisit if registration boilerplate stays noisy.
- **Switch MCP SDKs**: out of scope; the seam problem is ours, not the SDK's.

## Implementation Plan **[REQUIRED]**

1. Registry core + one pilot family (module tools) proving schema-from-struct parity
2. Migrate remaining families incrementally, deleting match arms/vec entries as they move
3. Delete `EXPECTED_TOOLS` bookkeeping tests once the registry is the single source
4. Fold in `From<ServiceError> for McpError` (closes MIMIR-I-0073)

Related: MIMIR-I-0070 (MapStateService) becomes cheaper after this — new map tools cost one registration each. MIMIR-I-0073 is absorbed by step 4.
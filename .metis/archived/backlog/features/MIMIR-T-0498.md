---
id: add-campaign-level-document
level: task
title: "Add campaign-level document support to MCP tools"
short_code: "MIMIR-T-0498"
created_at: 2026-01-30T04:20:36.362186+00:00
updated_at: 2026-01-30T04:45:47.673905+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Add campaign-level document support to MCP tools

## Objective

Expose campaign-level document operations through the MCP tools so users can create and manage documents that belong to the campaign rather than a specific module (e.g. world lore, session notes, campaign-wide references).

The database, models, DAL, and service layer already fully support this (`module_id` is nullable). Only the MCP tool layer and plugin documentation need updating.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `list_documents` works without `module_id` — returns campaign-level docs (requires active campaign)
- [ ] `create_document` works without `module_id` — creates a campaign-level doc
- [ ] Existing module-scoped calls continue working unchanged
- [ ] Plugin README updated with campaign-level document examples
- [ ] mimir-dm skill updated with campaign-level document workflow
- [ ] Analysis skills (continuity-check, npc-network, session-prep) updated to include campaign-level docs

## Implementation Notes

### Code Changes

**`crates/mimir-mcp/src/tools/document.rs`**:
1. `list_documents_tool()` — Make `module_id` optional. Remove from `required`.
2. `list_documents()` handler — Branch: with `module_id` use `list_for_module()`, without use `list_for_campaign()`.
3. `create_document_tool()` — Make `module_id` optional. Remove from `required`.
4. `create_document()` handler — Branch: with `module_id` use `CreateDocumentInput::for_module()`, without use `CreateDocumentInput::for_campaign()`.

No changes to `handler.rs` — tool names stay the same.

### Plugin Documentation Updates

**HIGH**: `README.md`, `skills/mimir-dm/SKILL.md`
**MEDIUM**: `skills/continuity-check/SKILL.md`, `skills/npc-network/SKILL.md`, `skills/session-prep/SKILL.md`, `commands/create-module.md`, `skills/mimir-dm/examples/create-module-workflow.md`
**LOW**: `skills/pressure-test/SKILL.md`

### No Backend Changes Needed
- Database schema already has nullable `module_id`
- Model has `module_id: Option<String>`
- DAL has `list_campaign_level_documents()`
- Service has `list_for_campaign()` and `CreateDocumentInput::for_campaign()`

## Status Updates

### Session 1
- Made `module_id` optional in `list_documents` and `create_document` tool definitions and handlers
- `list_documents` without `module_id` calls `service.list_for_campaign()` (requires active campaign)
- `create_document` without `module_id` calls `CreateDocumentInput::for_campaign()`
- Updated README.md, mimir-dm SKILL.md, continuity-check, npc-network, session-prep skills
- All compile clean
---
id: mcp-server-for-campaign-authoring
level: initiative
title: "MCP Server for Campaign Authoring"
short_code: "MIMIR-I-0038"
created_at: 2026-01-05T18:04:49.033858+00:00
updated_at: 2026-01-05T20:21:54.578196+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: mcp-server-for-campaign-authoring
---

# MCP Server for Campaign Authoring Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

Implement an MCP (Model Context Protocol) server that exposes Mimir's campaign management functionality, enabling agentic AI systems (Claude Code, etc.) to interact with the backend for authoring, organizing, and reviewing campaigns.

This enables AI-assisted campaign creation, content review, organization, and editing workflows directly through tools like Claude Code, positioning Mimir as an AI-native TTRPG tool.

## Goals & Non-Goals

**Goals:**
- MCP server binary using rust-mcp-sdk with stdio transport
- Document authoring tools (list, read, write, create from template)
- Character/NPC management tools (create, list, assign to modules)
- Campaign context tools (select campaign, list campaigns)
- Module info tools (list, get details)
- Claude Code skill/plugin with usage guidance

**Non-Goals:**
- Full catalog search/lookup (future phase)
- Session management tools (future phase)
- Combat tracking or real-time play tools
- HTTP transport (stdio only for MVP)

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design

### Tech Stack
- **MCP SDK**: rust-mcp-sdk (https://github.com/rust-mcp-stack/rust-mcp-sdk)
- **Transport**: stdio for Claude Code integration
- **Database**: Shared SQLite with Mimir app (WAL mode)

### Crate Structure
```
crates/mimir-dm-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point, stdio transport
│   ├── lib.rs               # Library exports
│   ├── server.rs            # MCP server setup
│   ├── handler.rs           # ServerHandler implementation
│   ├── context.rs           # Active campaign context
│   ├── error.rs             # MCP-specific errors
│   └── tools/
│       ├── mod.rs
│       ├── campaign.rs      # list_campaigns, set_active_campaign
│       ├── document.rs      # list, read, write, create_from_template
│       ├── character.rs     # list, create_npc, create_character, assign_to_module
│       └── module.rs        # list_modules, get_module_details
```

### MVP Tools (14 total)
- **Campaign**: set_active_campaign, list_campaigns, get_campaign_details
- **Document**: list_documents, read_document, write_document, create_document_from_template
- **Character**: list_characters, create_npc, create_character, get_character, assign_npc_to_module
- **Module**: list_modules, get_module_details

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

{Alternative approaches and why they were rejected}

## Implementation Plan

### Phase 1: MCP Server Foundation
- Create crate with Cargo.toml, add to workspace
- Set up MCP server with stdio transport
- Implement McpContext for campaign state management
- Add error types
- Implement list_campaigns, set_active_campaign, get_campaign_details

### Phase 2: Document Authoring Tools
- list_documents (filter by level, module, completion)
- read_document (file read with frontmatter)
- write_document (file write, mark completed)
- create_document_from_template

### Phase 3: Character/NPC Tools
- list_characters (filter by type)
- create_npc (simplified creation)
- create_character (full D&D 5e validation)
- get_character (with version history)
- assign_npc_to_module

### Phase 4: Module Tools & Polish
- list_modules, get_module_details
- Comprehensive error messages
- Tests

### Phase 5: Claude Code Integration
- Create plugin structure
- Write SKILL.md with usage guidance
- Test with Claude Code
- Documentation
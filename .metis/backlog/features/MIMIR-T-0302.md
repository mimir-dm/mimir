---
id: mcp-server-for-campaign-management
level: task
title: "MCP Server for Campaign Management"
short_code: "MIMIR-T-0302"
created_at: 2026-01-03T15:21:55.775929+00:00
updated_at: 2026-01-03T15:21:55.775929+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# MCP Server for Campaign Management

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

Implement an MCP (Model Context Protocol) server that exposes Mimir's campaign management functionality, enabling agentic AI systems (Claude Code, etc.) to interact with the backend for authoring, organizing, and reviewing campaigns.

## Backlog Item Details

### Type
- [ ] Bug - Production issue that needs fixing
- [x] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [x] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Business Justification
- **User Value**: Enables AI-assisted campaign creation, content review, organization, and editing workflows directly through tools like Claude Code
- **Business Value**: Differentiating feature that leverages AI tooling for enhanced DM productivity; positions Mimir as an AI-native TTRPG tool
- **Effort Estimate**: L

## Acceptance Criteria

- [ ] MCP server binary/process that can be launched alongside or integrated with Mimir
- [ ] Campaign CRUD tools (list, create, read, update, delete campaigns)
- [ ] Module management tools (list, create, read, update modules within campaigns)
- [ ] Session management tools (create, list, read sessions)
- [ ] NPC and item tools for module content
- [ ] Character roster tools (list characters, assign to campaigns)
- [ ] Search/query tools for finding content across campaigns
- [ ] Proper error handling with meaningful messages for agents
- [ ] Documentation for configuring MCP server in Claude Code / other clients
- [ ] Claude Code skill/plugin with usage guidance and domain context

## Potential Tool Categories

### Campaign Management
- `list_campaigns` - List all campaigns with summary info
- `read_campaign` - Get full campaign details
- `create_campaign` - Create new campaign
- `update_campaign` - Modify campaign metadata

### Module Authoring
- `list_modules` - List modules in a campaign
- `read_module` - Get module content and structure
- `create_module` - Create new module
- `edit_module_content` - Update module markdown/content
- `list_module_npcs` / `create_module_npc` - NPC management
- `list_module_items` / `create_module_item` - Item management

### Session & Play
- `list_sessions` - List campaign sessions
- `create_session` - Create new session
- `read_session` - Get session details and notes

### Reference & Search
- `search_campaign_content` - Full-text search across campaign
- `lookup_rule` / `lookup_monster` / `lookup_spell` - Catalog lookups

## Claude Code Skill/Plugin

A companion skill to teach Claude how to effectively use the MCP tools:

### Skill Content
- **Domain context**: What Mimir is, campaign/module/session hierarchy, TTRPG concepts
- **Workflow guidance**: Common patterns for authoring modules, organizing content, reviewing campaigns
- **Best practices**: Naming conventions, content structure, markdown formatting for modules
- **Tool usage examples**: When to use which tools, typical sequences for tasks

### Use Cases the Skill Enables
- "Help me flesh out this campaign setting"
- "Review my module for balance issues"
- "Create session notes from my outline"
- "Organize my NPCs by location"
- "Generate encounter ideas based on party level"

## Implementation Notes

### Technical Approach
- Rust MCP server using `rmcp` or similar crate
- Leverage existing service layer from mimir-dm-core
- stdio transport for local Claude Code integration
- Consider HTTP/SSE transport for remote access later

### Dependencies
- Stable service layer APIs in mimir-dm-core
- MCP SDK/library for Rust

### Considerations
- Read vs write permissions model
- Campaign-scoped operations (select active campaign context)
- Rate limiting for expensive operations

## Status Updates **[REQUIRED]**

*To be added during implementation*
---
id: pass-1-13-write-ai-assistant-how
level: task
title: "Pass 1.13: Write AI assistant how-to section"
short_code: "MIMIR-T-0609"
created_at: 2026-03-13T13:50:26.634336+00:00
updated_at: 2026-03-13T14:19:27.528889+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.13: Write AI assistant how-to section

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Write the AI assistant how-to section: a guide for using Mimir with Claude Code via the MCP plugin.

## Scope

Write `docs/src/how-to/ai-assistant/README.md` — a user-facing guide for using Mimir with Claude Code.

### Page Content

1. **What the MCP integration does** — Mimir includes a sidecar MCP server that exposes 60+ tools to Claude Code, enabling AI-assisted campaign management via natural language
2. **Prerequisites** — Claude Code installed, Mimir running
3. **Setup** — How the MCP server starts automatically as a Tauri sidecar (no manual config needed for app users). For standalone use: `MIMIR_DATABASE_PATH` env var
4. **Available commands** — The 5 slash commands from the `mimir-dm` plugin: `/mimir-campaigns`, `/create-module`, `/search-monsters`, `/search-spells`, `/generate-map`
5. **Example workflows** — Brief examples of what you can ask: "create a campaign", "search for CR 5 monsters", "generate a forest map", "add a +1 longsword to character inventory"
6. **Available skills** — List the 7+ specialized skills (encounter-balance, loot-audit, session-prep, etc.)
7. **Limitations** — Note that MCP tools modify the database directly; no undo. Link to export-campaign.md for backups.

### Verification Sources
- `crates/mimir-mcp/plugin/plugin.json` — plugin manifest with commands and skills
- `crates/mimir-mcp/plugin/README.md` — existing plugin documentation
- `crates/mimir-mcp/src/main.rs` — server configuration
- `tauri.conf.json` — sidecar/externalBin configuration

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Page created at `docs/src/how-to/ai-assistant/README.md`
- [ ] Plugin commands and skills verified against actual `plugin.json`
- [ ] MCP server setup description matches actual architecture
- [ ] Example workflows are achievable with actual MCP tools
- [ ] Matches tone of existing how-to pages

## Status Updates

### 2026-03-13: Completed
Created `docs/src/how-to/ai-assistant/README.md`.

**Content:**
- MCP server overview (40+ tools via Tauri sidecar)
- Setup: plugin install command, database path auto-detection + override
- 5 slash commands verified against `crates/mimir-mcp/plugin/commands/`
- 8 skills verified against `crates/mimir-mcp/plugin/skills/*/SKILL.md`
- Example workflows covering campaign setup, search, mapgen, session prep, character management
- Tool categories (8 areas)
- Important notes: no undo, backup recommendation, DM-in-control philosophy
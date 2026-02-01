---
id: improve-mimir-claude-code-plugin
level: task
title: "Improve Mimir Claude Code Plugin Quality"
short_code: "MIMIR-T-0497"
created_at: 2026-01-29T15:00:24.573719+00:00
updated_at: 2026-01-29T15:43:26.520898+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Improve Mimir Claude Code Plugin Quality

## Objective

Implement all 12 recommendations from the plugin-validator, skill-reviewer, and manual review of the Mimir Claude Code plugin at `crates/mimir-mcp/plugin/`. Fix structural issues, add frontmatter to all skills, fix incorrect examples, improve content quality, and add progressive disclosure.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Plugin may not be discoverable (no .claude-plugin/), skills may not trigger (no frontmatter), NPC example has incorrect API calls, tool tables duplicate MCP metadata
- **Benefits of Fixing**: Plugin works correctly with Claude Code, skills trigger reliably, examples produce working tool calls, content is lean and workflow-focused
- **Risk Assessment**: Low risk — all changes are to markdown/JSON config files, no code changes

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.claude-plugin/` directory exists with `plugin.json` inside
- [ ] MCP config exists in one place only (no duplication)
- [ ] All 7 SKILL.md files have YAML frontmatter with name and description
- [ ] NPC example uses only valid `edit_character` parameters
- [ ] mimir-dm SKILL.md body focuses on workflows, not tool tables
- [ ] session-prep references the other 5 analysis skills
- [ ] encounter-balance and loot-audit have `references/` subdirectories with extracted tables
- [ ] encounter-balance XP tables cover levels 1-20 and CR 0-30
- [ ] Document types in mimir-dm SKILL.md match actual MCP tool schema
- [ ] All skill bodies use imperative voice consistently
- [ ] Section ordering normalized: Purpose → Process → Output Format → Interactive Mode
- [ ] Output templates use text labels instead of emoji

## Implementation Checklist

### Critical (1-2)
1. [x] Create `.claude-plugin/` directory, move `plugin.json` into it
2. [x] Remove `.mcp.json` (keep MCP config only in plugin.json), verify field name is correct

### Major (3-8)
3. [x] Add YAML frontmatter to all 7 SKILL.md files, remove duplicate description from body
4. [x] Fix `examples/create-npc-workflow.md` Step 3 — remove `traits`, `ideals`, `bonds`, `flaws` parameters
5. [x] Rewrite mimir-dm SKILL.md body — replace tool tables with workflow guidance, add pointers to examples/
6. [x] Add cross-skill references in session-prep SKILL.md
7. [x] Create `encounter-balance/references/5e-encounter-math.md` with complete tables (levels 1-20, CR 0-30), extract tables from SKILL.md
8. [x] Create `loot-audit/references/5e-treasure-guidelines.md`, extract tables from SKILL.md

### Minor (9-12)
9. [x] Verify and fix document types list in mimir-dm SKILL.md against MCP tool schema
10. [x] Fix imperative voice throughout all skills (remove "you must", use direct imperatives)
11. [x] Normalize section ordering across all 6 analysis skills
12. [x] Replace emoji in output templates with text labels ([OK], [WARNING], [CRITICAL])

## Status Updates

### Session 1 — All items complete
- [x] Item 1: Created `.claude-plugin/` dir, moved `plugin.json`
- [x] Item 2: Removed `.mcp.json`
- [x] Item 3: Added YAML frontmatter to all 7 SKILL.md files
- [x] Item 4: Fixed NPC example — removed invalid `edit_character` params (traits/ideals/bonds/flaws)
- [x] Item 5: Rewrote mimir-dm SKILL.md — workflow-focused body, correct document types
- [x] Item 6: Added cross-skill references in session-prep
- [x] Item 7: Created `encounter-balance/references/5e-encounter-math.md` with levels 1-20, CR 0-30
- [x] Item 8: Created `loot-audit/references/5e-treasure-guidelines.md` with full guidelines
- [x] Item 9: Fixed document types to match MCP schema (backstory, read_aloud, dm_notes, description, custom)
- [x] Item 10: Imperative voice throughout all skills
- [x] Item 11: Normalized section ordering (Purpose → Process → Output → Interactive)
- [x] Item 12: Replaced emoji with text labels ([OK], [WARNING], [CRITICAL])
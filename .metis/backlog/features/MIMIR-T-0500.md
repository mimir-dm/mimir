---
id: plugin-documentation-and-agent
level: task
title: "Plugin Documentation and Agent Behavior Improvements"
short_code: "MIMIR-T-0500"
created_at: 2026-01-30T19:51:18.507266+00:00
updated_at: 2026-01-31T13:43:48.916478+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Plugin Documentation and Agent Behavior Improvements

## Objective

Improve the Claude Code plugin skills and documentation so agents produce reliable, correct results when using Mimir MCP tools. Currently skills lack defensive patterns, leading to silent failures and incomplete operations.

## Dependencies

- MIMIR-T-0499 (MCP Tool Coverage Gaps) — new tools need corresponding plugin docs
- Plugin files live under `crates/mimir-mcp/plugin/`

## 1. Systematic Agent Behavior Patterns

These patterns are missing across ALL skills and need to be added as standard practice.

### Read-Before-Edit
Every skill that instructs agents to edit documents/characters/modules must include:
> "Always call `get_character` / `read_document` / `get_module_details` before making edits to confirm current state."

**Affected skills:** mimir-dm, session-prep, continuity-check, npc-network, all example workflows

### Pre-Check Before Create
Before creating entities, verify preconditions:
- Campaign is active (`list_campaigns` + `set_active_campaign`)
- Module exists before adding monsters/items/documents
- Character exists before adding inventory

**Affected skills:** mimir-dm, create-module command, all example workflows

### Error Handling Guidance
Add standard error handling instructions:
- "If a tool call fails, report the error to the user rather than silently continuing"
- "If `search_monsters`/`search_items` returns no results, try alternate names or broader search"
- "If `add_monster_to_module` fails, verify the monster name matches the catalog exactly (case-sensitive)"

**Affected skills:** All skills

### Catalog Exact-Match Rules
Search tools return partial matches. Skills must instruct agents to:
- Use `search_monsters`/`search_items`/`search_spells` first
- Pick the exact matching name from results
- Use that exact name string in `add_monster_to_module` / `add_item_to_module` / `add_item_to_character`

**Affected skills:** mimir-dm, encounter-balance, loot-audit, session-prep

## 2. Skill-Specific Improvements

### `mimir-dm/SKILL.md` (PRIMARY)
- Add "Getting Started" section: check campaign, set active, verify modules
- Add campaign-level document workflow (partially done in MIMIR-T-0498)
- Add character management workflow: create → set ability scores → set currency → add inventory
- Add level-up workflow once `level_up_character` tool exists
- Document the full character creation checklist (race, background, ability scores, currency, inventory, roleplay traits)

### `encounter-balance/SKILL.md`
- Add step: verify module exists and has monsters before analysis
- Add guidance: cross-reference `search_monsters` CR data with party level
- Add: "If monster not found in catalog, note it as homebrew"

### `loot-audit/SKILL.md`
- Add: read module items AND character inventories for full picture
- Add: use `search_items` to verify rarity of each item
- Add: include campaign-level documents in loot analysis

### `continuity-check/SKILL.md`
- Already partially updated for campaign-level docs
- Add: cross-reference NPC names across modules for consistency
- Add: verify character references in documents match actual characters

### `npc-network/SKILL.md`
- Already partially updated for campaign-level docs
- Add: use `list_characters` with faction filter to build faction graph
- Add: cross-reference NPC locations with module locations

### `session-prep/SKILL.md`
- Already partially updated for campaign-level docs
- Add: verify all referenced monsters/items exist in catalog
- Add: check character inventories are populated
- Add: verify encounter balance for planned encounters

### `pressure-test/SKILL.md`
- Add: verify all documents, monsters, items are catalog-valid
- Add: check for orphaned NPCs (not assigned to modules)
- Add: check for empty modules

## 3. README.md Updates

- Document all new tools from MIMIR-T-0499 as they are implemented
- Add "Character Management" section with full lifecycle
- Add "Campaign Management" section
- Update tool parameter tables for modified tools
- Add "Common Patterns" section covering read-before-edit, pre-checks, exact-match

## 4. Example Workflow Updates

### `create-module-workflow.md`
- Add pre-check: verify active campaign
- Add read-before-edit before any edits
- Add catalog search before add_monster/add_item

### New workflow: `character-creation-workflow.md`
- Full character creation: create → ability scores → currency → race/background → inventory → roleplay traits

### New workflow: `level-up-workflow.md`
- Level up flow: get_character → choose class → level_up_character → verify result

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All skills include read-before-edit instructions
- [ ] All skills include pre-check-before-create instructions
- [ ] All skills include error handling guidance
- [ ] Catalog exact-match pattern documented in all relevant skills
- [ ] mimir-dm SKILL.md has full character management workflow
- [ ] README.md documents all new/updated tools
- [ ] At least 2 new example workflows added (character creation, level-up)
- [ ] Campaign-level document workflow complete in all relevant skills

## Status Updates

*To be added during implementation*
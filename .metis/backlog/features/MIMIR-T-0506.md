---
id: homebrew-monster-creation-with
level: task
title: "Homebrew monster creation with catalog cloning"
short_code: "MIMIR-T-0506"
created_at: 2026-01-31T13:48:50.364100+00:00
updated_at: 2026-01-31T13:48:50.364100+00:00
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

# Homebrew monster creation with catalog cloning

## Objective

Allow DMs to create custom homebrew monsters/creatures within a campaign. Users should be able to build monsters from scratch or clone an existing catalog monster (e.g., a Goblin) as a starting point and reskin/modify its stats, abilities, and flavor. Homebrew monsters must survive the full campaign import/export round trip.

## Backlog Details

- **Type**: Feature
- **Priority**: P2 - Medium
- **Effort**: L

### Business Justification
- **User Value**: DMs constantly reskin and modify monsters — a "Shadow Goblin" that's a Goblin with necrotic damage resistance, or a completely custom creature. Currently there's no way to use monsters that aren't in the 5etools catalog. Reskinning is one of the most common DM workflows.
- **Effort Estimate**: L — requires new DB tables, DAL, service, Tauri commands, and a monster editor UI with stat block preview.

## Acceptance Criteria

- [ ] DM can create a new homebrew monster from scratch within a campaign, specifying: name, size, type, alignment, AC, HP (dice formula), speed, ability scores, saving throws, skills, damage resistances/immunities/vulnerabilities, condition immunities, senses, languages, CR, traits, actions, reactions, legendary actions, and description/lore
- [ ] DM can "clone from catalog" — search the monster catalog, select a creature, and create a homebrew copy pre-populated with that monster's full stat block for editing
- [ ] Cloned monsters track their origin (`cloned_from` catalog reference) but are fully independent
- [ ] Homebrew monsters appear in module encounter builders alongside catalog monsters, distinguished with a homebrew badge/indicator
- [ ] Homebrew monsters can be used in encounters, initiative trackers, and any other place catalog monsters appear
- [ ] Campaign export includes all homebrew monsters in the export payload
- [ ] Campaign import re-creates homebrew monsters, handling duplicates gracefully (skip or update)
- [ ] Deleting a homebrew monster that's referenced in encounters warns the user and handles cleanup
- [ ] Optional: custom token image upload for homebrew monsters

## Implementation Notes

### Data Model Considerations
- Campaign-scoped table (e.g., `campaign_homebrew_monsters`) similar to the homebrew items approach in MIMIR-T-0505
- Each homebrew monster stores its full data blob (same JSON structure as catalog monsters) plus `campaign_id` and optional `cloned_from_catalog_id`
- Monster search/listing in encounter builders should union across catalog AND campaign homebrew monsters
- The data blob should use the same 5etools-compatible JSON schema so rendering code (stat blocks, etc.) works identically for catalog and homebrew monsters

### Clone Workflow
1. User clicks "Create Homebrew Monster" or "Clone from Catalog"
2. If cloning: search catalog → select monster → pre-populate editor with full stat block
3. User edits any field — name, stats, abilities, traits, actions, etc.
4. Save creates a campaign-scoped homebrew entry
5. Stat block preview updates live during editing

### Import/Export Round Trip
- Export format needs a `homebrew_monsters` section in the campaign JSON
- Import reads this section and creates/updates homebrew monsters in the target campaign
- Must handle: monster exists in target (update vs skip), monster referenced by encounters/modules (preserve associations)
- Token images: either embed as base64 in export or reference by path with fallback

### Shared Architecture with MIMIR-T-0505
- Both homebrew weapons and monsters follow the same pattern: campaign-scoped table, clone-from-catalog, round-trip export/import
- Consider a shared `campaign_homebrew` abstraction or at minimum consistent schema patterns between the two
- The frontend clone workflow (search catalog → select → edit) should feel identical for both

### MCP Tool Considerations
- MCP tools for campaign management should expose homebrew monster CRUD so AI-assisted campaign building can create custom creatures programmatically

## Status Updates

*To be added during implementation*
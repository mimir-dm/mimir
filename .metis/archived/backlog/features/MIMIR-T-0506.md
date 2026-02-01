---
id: homebrew-monster-creation-with
level: task
title: "Homebrew monster creation with catalog cloning"
short_code: "MIMIR-T-0506"
created_at: 2026-01-31T13:48:50.364100+00:00
updated_at: 2026-02-01T03:47:43.725660+00:00
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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

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

## Design Decisions

- **Clone-only + JSON editor** — no structured from-scratch form. Monster stat blocks are too complex for a fully structured form. Users clone from catalog and edit the JSON.
- **Reuse `MonsterStatsPanel`** for live preview of the stat block while editing
- **HomebrewTab gets sub-tabs** ("Items" | "Monsters") rather than a separate dashboard tab

## Implementation Plan

### 1. Migration — `campaign_homebrew_monsters` table

**New:** `crates/mimir-core/migrations/025_campaign_homebrew_monsters/up.sql` + `down.sql`

```sql
CREATE TABLE campaign_homebrew_monsters (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cr TEXT,
    creature_type TEXT,
    size TEXT,
    data TEXT NOT NULL,
    cloned_from_name TEXT,
    cloned_from_source TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, name)
);
```

Then `diesel print-schema` to regenerate `schema.rs`.

### 2. Model + DAL

**New:** `models/campaign/campaign_homebrew_monster.rs` — CampaignHomebrewMonster, New, Update structs
**New:** `dal/campaign/campaign_homebrew_monster.rs` — full CRUD + get_by_name + delete_all
**Modify:** `models/campaign/mod.rs`, `dal/campaign/mod.rs` — register + re-export

### 3. Archive Export/Import

**Modify:** `services/archive.rs`
- Add `homebrew_monsters` to `ArchiveData` (`#[serde(default)]`)
- Add count to `ArchiveCounts`
- Collect in `collect_campaign_data`, import with remapped `campaign_id`

### 4. Tauri Commands

**New:** `commands/homebrew_monster.rs` — list, get, create, update, delete
**Modify:** `commands/mod.rs`, `main.rs`

### 5. MCP Tools

**New:** `mimir-mcp/src/tools/homebrew_monster.rs` — 5 tools
**Modify:** `tools/mod.rs`, `handler.rs` — register + update `EXPECTED_TOOLS`

### 6. Frontend Service

**New:** `services/HomebrewMonsterService.ts` — types + CRUD
**Modify:** `utils/dataEvents.ts` — add monster events

### 7. Frontend UI

**Modify:** `HomebrewTab.vue` — add sub-tab bar ("Items" | "Monsters")
**New:** `HomebrewMonstersSubTab.vue`:
- List/detail split layout (same as items)
- Cards show: name, CR, size, creature type
- Detail pane reuses `MonsterStatsPanel`
- "Clone from Catalog" button → search modal using `search_monsters`
- Create/edit modal: name + JSON textarea + live MonsterStatsPanel preview
- Delete with module-usage warning

### Files Summary

| File | Action |
|------|--------|
| `migrations/025_.../up.sql` + `down.sql` | New |
| `schema.rs` | Regenerate |
| `models/campaign/campaign_homebrew_monster.rs` | New |
| `models/campaign/mod.rs` | Modify |
| `dal/campaign/campaign_homebrew_monster.rs` | New |
| `dal/campaign/mod.rs` | Modify |
| `services/archive.rs` | Modify |
| `commands/homebrew_monster.rs` | New |
| `commands/mod.rs` | Modify |
| `main.rs` | Modify |
| `mimir-mcp/src/tools/homebrew_monster.rs` | New |
| `mimir-mcp/src/tools/mod.rs` | Modify |
| `mimir-mcp/src/handler.rs` | Modify |
| `services/HomebrewMonsterService.ts` | New |
| `utils/dataEvents.ts` | Modify |
| `HomebrewTab.vue` | Modify |
| `HomebrewMonstersSubTab.vue` | New |

### Verification

1. `diesel migration run` — applies
2. `cargo check -p mimir-core` / `mimir` / `mimir-mcp` — compiles
3. `cargo test -p mimir-mcp` — EXPECTED_TOOLS passes
4. `npx vue-tsc --noEmit` — no new errors
5. Clone monster from catalog, edit JSON, save, see stat block

## Status Updates

*To be added during implementation*
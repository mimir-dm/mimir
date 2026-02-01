---
id: homebrew-weapon-creation-with
level: task
title: "Homebrew weapon creation with catalog cloning"
short_code: "MIMIR-T-0505"
created_at: 2026-01-31T13:48:49.869748+00:00
updated_at: 2026-01-31T21:16:46.135741+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Homebrew item creation with catalog cloning

## Objective

Allow DMs to create custom homebrew items (weapons, armor, potions, wondrous items, mundane gear — any equipment type) within a campaign. Users should be able to create items from scratch or clone an existing catalog item as a starting point and modify its properties. Homebrew items must survive the full campaign import/export round trip.

## Backlog Details

- **Type**: Feature
- **Priority**: P2 - Medium
- **Effort**: L

### Business Justification
- **User Value**: DMs frequently need custom or reskinned items — a "Moonblade" that's mechanically a +1 Longsword, a custom potion, or a completely novel wondrous item. Currently there's no way to add items that aren't in the 5etools catalog.
- **Effort Estimate**: L — requires new DB table + migration, DAL, service, Tauri commands, export/import changes, and frontend UI.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] DM can create a new homebrew item from scratch within a campaign, specifying name, item_type, damage dice, damage type, AC, weight, properties, rarity, attunement, and description/entries
- [ ] DM can "clone from catalog" — search the item catalog, select any item, and create a homebrew copy pre-populated with that item's stats for editing
- [ ] Cloned items track their origin (`cloned_from_name`, `cloned_from_source`) but are independent — editing the homebrew doesn't affect the catalog
- [ ] Homebrew items appear in the "Add equipment" search alongside catalog items, distinguished with a homebrew badge/indicator
- [ ] Homebrew items can be assigned to character inventories the same way catalog items can
- [ ] Campaign export includes all homebrew items in the export payload
- [ ] Campaign import re-creates homebrew items, handling duplicates gracefully (skip or update)
- [ ] Deleting a homebrew item that's in a character's inventory warns the user and handles cleanup

## Design Decisions

### Source Convention
- All homebrew items use source `"HB"` in both the homebrew table and character inventory references
- Campaign scoping is handled by the `campaign_id` column, not the source string
- UNIQUE constraint on `(campaign_id, name)` prevents duplicate names within a campaign
- Inventory lookup: check `campaign_homebrew_items` (by name + campaign_id) first, then fall back to global `items` catalog

### Import/Export Stability
- Inventory references homebrew items by `name|"HB"` — strings, not UUIDs
- On import, `campaign_id` is remapped (new UUID generated), but name+source strings are stable
- Homebrew items are re-created under the new campaign_id; inventory entries match by name as before

## Implementation Plan

### 1. Migration — `campaign_homebrew_items` table
```sql
CREATE TABLE campaign_homebrew_items (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    item_type TEXT,
    rarity TEXT,
    data TEXT NOT NULL,           -- Full JSON blob (same structure as catalog items)
    cloned_from_name TEXT,        -- Original catalog item name (if cloned)
    cloned_from_source TEXT,      -- Original catalog item source (if cloned)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, name)
);
```

### 2. DAL + Models
- `NewHomebrewItem`, `HomebrewItem`, `UpdateHomebrewItem` models
- CRUD: `insert_homebrew_item`, `get_homebrew_item`, `list_homebrew_items(campaign_id)`, `update_homebrew_item`, `delete_homebrew_item`
- `get_homebrew_item_by_name(campaign_id, name)` for inventory lookup

### 3. Service Layer
- `HomebrewItemService` with create, update, delete, list, clone-from-catalog
- Clone helper: fetch catalog item by name+source, copy its data blob into a new homebrew entry

### 4. Tauri Commands
- `create_homebrew_item`, `update_homebrew_item`, `delete_homebrew_item`, `list_homebrew_items`
- Modify `get_item_by_name` to check homebrew table first (needs campaign_id context)
- Modify `search_items` to union homebrew results

### 5. Export/Import (archive.rs)
- Add `homebrew_items: Vec<HomebrewItem>` to `ArchiveData`
- Export: collect homebrew items for campaign
- Import: insert homebrew items with remapped campaign_id

### 6. Frontend
- Homebrew creation form (or clone-from-catalog flow) in InventoryManager
- "HB" badge on homebrew items in search results and inventory display
- Edit/delete homebrew items

### 7. MCP Tools
- Expose homebrew CRUD for AI-assisted campaign building

## Key Files

| File | Action |
|------|--------|
| `crates/mimir-core/migrations/` | New migration for `campaign_homebrew_items` table |
| `crates/mimir-core/src/schema.rs` | Auto-generated after migration (diesel print-schema) |
| `crates/mimir-core/src/models/campaign/` | New `homebrew_item.rs` module + register in `mod.rs` |
| `crates/mimir-core/src/dal/campaign/` | New `homebrew_item.rs` DAL + register in `mod.rs` |
| `crates/mimir-core/src/services/` | New `homebrew_item.rs` service + register in `mod.rs` |
| `crates/mimir-core/src/services/archive.rs` | Add `homebrew_items` to `ArchiveData`, export, and import |
| `crates/mimir/src/commands/` | New `homebrew.rs` Tauri commands + register in `main.rs` |
| `crates/mimir/src/commands/catalog.rs` | Modify `get_item_by_name` to check homebrew first |
| `crates/mimir/frontend/src/features/characters/components/InventoryManager.vue` | Add homebrew create/clone UI, union homebrew into search |
| `crates/mimir/frontend/src/features/characters/components/sheet/EquipmentSection.vue` | HB badge display |
| `crates/mimir/frontend/src/services/` | New `HomebrewService.ts` |

## Status Updates

### Session 1 — Backend Implementation
- Created migration `024_campaign_homebrew_items/` with up.sql and down.sql
- Updated `schema.rs` with new table definition, joinable, and allow_tables entry
- Created model: `models/campaign/campaign_homebrew_item.rs` — CampaignHomebrewItem, New, Update structs
- Created DAL: `dal/campaign/campaign_homebrew_item.rs` — full CRUD + get_by_name
- Registered both in respective `mod.rs` files
- Updated `services/archive.rs`:
  - Added `homebrew_items` to ArchiveData, ArchiveCounts
  - Added collection in `collect_campaign_data`
  - Added `import_homebrew_items` method
  - Updated manifest counts in both export and import
  - Used `#[serde(default)]` for backward compatibility with older archives
- Created Tauri commands: `commands/homebrew.rs` — list, get, get_by_name, create, update, delete
- Registered commands in `commands/mod.rs` and `main.rs`
- Created `frontend/src/services/HomebrewService.ts` — full TypeScript service
- **All code compiles clean** (`cargo check --package mimir` passes)
- **No new test failures** (48 pre-existing failures, 0 homebrew-related)

### Session 1 — Frontend & Catalog Integration
- Modified `commands/catalog.rs::get_item_by_name` — added optional `campaign_id` param, checks homebrew table first when source="HB"
- Updated `EquipmentSection.vue` — passes `campaignId` to `get_item_by_name` for homebrew lookup, added HB badge
- Updated `InventoryManager.vue` — merges homebrew items into search results alongside catalog items, HB badge in search and inventory list
- Created `HomebrewService.ts` — full TypeScript service with CRUD operations
- Registered homebrew events in `dataEvents.ts`
- **Rust: `cargo check --package mimir` passes clean**
- **TypeScript: `vue-tsc --noEmit` passes clean** (1 pre-existing error in ModuleService)
- **No new test failures**

### Not Implemented (scope decisions)
- Frontend homebrew creation/edit form UI (DM-facing creation workflow) — deferred to a follow-up task
- Clone-from-catalog flow UI — deferred to a follow-up task
- MCP tools for homebrew CRUD — deferred to a follow-up task
- Delete-with-inventory-warning — deferred to follow-up

### Session 2 — Homebrew Dashboard Tab & Seeder
- Created `HomebrewTab.vue` — full dashboard tab with list/detail layout, create/edit/delete modals
- Uses `ItemDetailBlock` for rich item rendering (type, rarity, stats, description) instead of raw JSON
- Raw JSON available under collapsible toggle for inspection
- Registered tab in `useDashboardState.ts` and `router/index.ts`
- Switched to global button/modal/form styles from `buttons.css`, `modals.css`, `forms.css`
- Added padding to match other dashboard tabs (NPCsTab pattern)
- Added 4 homebrew items to dev seeder with 3 assigned to characters as equipped inventory
- **Committed as `3170753`** — 21 files, 1663 insertions

### Remaining Work — Plan for Session 3

**1. MCP Tools for Homebrew CRUD**
- New file: `crates/mimir-mcp/src/tools/homebrew.rs` — 5 tools (list, get, create, update, delete)
- Register in `tools/mod.rs` and `handler.rs` (get_tools + execute_tool + EXPECTED_TOOLS test)
- Uses existing DAL functions directly via `ctx.db()`

**2. Clone-from-Catalog in HomebrewTab**
- Add "Clone from Catalog" button in header, opens modal with catalog search input
- Selecting a catalog item pre-fills create form (name, type, rarity, data JSON, cloned_from fields)
- Add `cloned_from_name`/`cloned_from_source` to form ref, pass to `HomebrewService.create()`

**3. Delete-with-Inventory-Warning**
- Before delete confirmation, check if any characters use the item (client-side: list characters + check inventories for name+source="HB")
- If in use: show warning listing affected character names
- If not in use: simple confirmation (current behavior)

**Files:** `mimir-mcp/src/tools/homebrew.rs` (new), `mimir-mcp/src/tools/mod.rs`, `mimir-mcp/src/handler.rs`, `HomebrewTab.vue`

### Session 3 — MCP Tools, Clone-from-Catalog, Delete Warning
- Created `crates/mimir-mcp/src/tools/homebrew.rs` — 5 MCP tools (list, get, create, update, delete)
- Registered in `tools/mod.rs` and `handler.rs` (get_tools, execute_tool, EXPECTED_TOOLS)
- All 16 mimir-mcp tests pass (including tool count and route coverage)
- Updated `HomebrewTab.vue`:
  - Added "Clone from Catalog" button with search modal using debounced `search_items` Tauri command
  - Selecting a catalog result pre-fills create form with name, type, rarity, data JSON, and cloned_from fields
  - Added `cloned_from_name`/`cloned_from_source` to form ref, passed to `HomebrewService.create()`
  - Delete now checks character inventories for item_name + source="HB" before confirming
  - Shows warning listing affected character names if item is in use

### What IS Complete
1. Full database layer: migration, schema, model, DAL (CRUD + get_by_name)
2. Full service layer: archive export/import with backward compatibility
3. Full Tauri command layer: 6 commands registered
4. Full TypeScript service with typed API
5. Catalog integration: `get_item_by_name` checks homebrew first for HB source
6. Frontend display: HB badges in EquipmentSection and InventoryManager
7. Search integration: homebrew items appear alongside catalog items in Add Item search
8. Homebrew dashboard tab with CRUD UI, ItemDetailBlock rendering, and global style consistency
9. Dev seeder with 4 sample homebrew items (Oathkeeper, Blessed Talisman of Tyr, Whisper, Cragmaw Brew)
10. MCP tools: 5 homebrew CRUD tools registered and tested
11. Clone-from-catalog UI flow in HomebrewTab
12. Delete-with-inventory-warning in HomebrewTab
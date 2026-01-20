---
id: add-display-name-and-notes-fields
level: task
title: "Add display name and notes fields to module monsters"
short_code: "MIMIR-T-0350"
created_at: 2026-01-19T13:31:51.668807+00:00
updated_at: 2026-01-19T21:59:46.713085+00:00
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

# Add display name and notes fields to module monsters

Allow DMs to customize monsters added to modules by providing an optional display name (alias) and free-form notes field. This enables use cases like "use this goblin stat block but call it a frost wight" or "notes about thematic ability modifications for this campaign."

## Objective

Add `display_name` and `notes` fields to the `ModuleMonster` model, allowing DMs to customize how monsters appear and document any thematic modifications without altering the underlying catalog reference.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: DMs frequently reskin monsters for thematic purposes. A "goblin" might become a "frost wight" in a winter campaign while retaining the same stat block. Currently there's no way to track these customizations, forcing DMs to maintain separate notes.
- **Business Value**: Improves campaign authoring workflow completeness; reduces need for external note-taking tools.
- **Effort Estimate**: M (Medium) - Database migration, model updates, service layer, MCP tools, and UI changes across multiple files.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `ModuleMonster` struct includes optional `display_name: Option<String>` field
- [x] `ModuleMonster` struct includes optional `notes: Option<String>` field
- [x] Database migration adds `display_name` and `notes` columns to `module_monsters` table
- [x] `CreateModuleMonster` accepts optional `display_name` and `notes` parameters
- [x] `UpdateModuleMonster` allows updating `display_name` and `notes` fields
- [x] MCP `add_monster_to_module` tool accepts optional `display_name` and `notes` parameters
- [x] MCP `update_module_monster` tool can update `display_name` and `notes`
- [x] UI displays `display_name` (if set) instead of `monster_name` in monster lists
- [x] UI shows original `monster_name` as subtitle/tooltip when `display_name` is set
- [x] UI provides modal editing for `display_name` and `notes` fields
- [x] UI provides expandable/collapsible notes section for each monster (shown in MonsterStatsPanel)
- [x] All existing monsters continue to work (migration handles NULL values)

## Implementation Notes

### Technical Approach

#### 1. Database Migration
Create migration to add columns to `module_monsters` table:
```sql
ALTER TABLE module_monsters ADD COLUMN display_name TEXT;
ALTER TABLE module_monsters ADD COLUMN notes TEXT;
```

#### 2. Model Updates (`crates/mimir-dm-core/src/models/campaign/module_monsters.rs`)

Update `ModuleMonster` struct:
```rust
pub struct ModuleMonster {
    pub id: i64,
    pub module_id: i64,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    pub display_name: Option<String>,  // NEW
    pub notes: Option<String>,          // NEW
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Update `CreateModuleMonster`:
```rust
pub struct CreateModuleMonster {
    pub module_id: i64,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: Option<i32>,
    pub encounter_tag: Option<String>,
    pub display_name: Option<String>,  // NEW
    pub notes: Option<String>,          // NEW
}
```

Update `UpdateModuleMonster`:
```rust
pub struct UpdateModuleMonster {
    pub quantity: Option<i32>,
    pub encounter_tag: Option<String>,
    pub display_name: Option<String>,  // NEW
    pub notes: Option<String>,          // NEW
}
```

#### 3. Repository Updates (`crates/mimir-dm-core/src/repositories/module_monster_repository.rs`)

- Update `create()` to include new fields in INSERT
- Update `update()` to handle new fields
- Update `find_*` queries to SELECT new columns
- Update `FromRow` impl if using manual mapping

#### 4. Service Layer (`crates/mimir-dm-core/src/services/module_service.rs`)

- Pass through new fields in `add_monster_to_module()`
- Pass through new fields in `update_module_monster()`

#### 5. MCP Tool Updates (`crates/mimir-dm-mcp/src/tools/`)

**`add_monster_to_module` tool:**
- Add optional `display_name` parameter with description: "Custom display name for this monster (e.g., 'Frost Wight' when using goblin stats)"
- Add optional `notes` parameter with description: "DM notes about customizations or thematic changes"

**`update_module_monster` tool:**
- Add optional `display_name` parameter
- Add optional `notes` parameter

#### 6. Frontend Updates

**Types (`frontend/src/types/`):**
```typescript
interface ModuleMonster {
  id: number
  module_id: number
  monster_name: string
  monster_source: string
  quantity: number
  encounter_tag?: string
  display_name?: string  // NEW
  notes?: string         // NEW
}
```

**MonsterList component updates:**
- Display `display_name || monster_name` as primary name
- Show `monster_name` in smaller text/tooltip when display_name differs
- Add pencil icon to edit display_name inline
- Add expandable notes section (collapsed by default)
- Add textarea for notes editing

**Tauri commands:**
- Update `add_monster_to_module` command signature
- Update `update_module_monster` command signature

### Files to Modify

1. `crates/mimir-dm-core/src/models/campaign/module_monsters.rs` - Model structs
2. `crates/mimir-dm-core/src/repositories/module_monster_repository.rs` - Database operations
3. `crates/mimir-dm-core/src/services/module_service.rs` - Service methods
4. `crates/mimir-dm-mcp/src/tools/module_tools.rs` - MCP tool definitions
5. `crates/mimir-dm/src-tauri/src/commands/module_commands.rs` - Tauri commands
6. `crates/mimir-dm/frontend/src/types/module.ts` - TypeScript types
7. `crates/mimir-dm/frontend/src/features/campaigns/components/dashboard/ModulesTab.vue` - UI component
8. `crates/mimir-dm/frontend/src/services/moduleService.ts` - Frontend service

### Dependencies
- None - self-contained feature

### Risk Considerations
- **Migration safety**: NULL values for existing rows are acceptable; no data loss risk
- **UI complexity**: Notes section should be collapsible to avoid cluttering the monster list
- **Performance**: Notes could be large; consider lazy loading if performance issues arise

## Status Updates

### Session 1 - 2026-01-19

**Completed all implementation steps:**

1. **Database Migration** (045_add_module_monster_customization)
   - Created `up.sql` with `ALTER TABLE` to add `display_name` and `notes` columns
   - Created `down.sql` with SQLite table recreation pattern

2. **Schema Updates** (`schema.rs`)
   - Added `display_name -> Nullable<Text>` and `notes -> Nullable<Text>` to diesel table macro

3. **Model Updates** (`models/campaign/module_monsters.rs`)
   - Added fields to `ModuleMonster`, `NewModuleMonster`, `UpdateModuleMonster`, `ModuleMonsterWithData`
   - Updated `From<ModuleMonster> for ModuleMonsterWithData` impl

4. **Service Layer** (`services/module_monster_service.rs`)
   - Updated `add_monster()` signature to accept new fields
   - Updated `update_monster()` signature to accept new fields

5. **MCP Tools** (`mimir-dm-mcp/src/tools/campaign.rs`)
   - Added `display_name` and `notes` parameters to `add_monster_to_module` tool
   - Created new `update_module_monster` tool with all editable fields
   - Updated `AddMonsterResponse` and created `UpdateMonsterResponse`
   - Registered tools in `handler.rs` and `mod.rs`

6. **Tauri Commands** (`commands/campaign/module_monsters.rs`)
   - Updated `AddMonsterRequest` and `UpdateMonsterRequest` structs
   - Updated command handlers to pass new fields

7. **Frontend Types & UI**
   - Updated `MonsterWithData` interface in composable
   - Updated interfaces in `ModuleMonsters.vue` and `TokenPalette.vue`
   - Updated `ModulesTab.vue` to display `display_name` with original name as subtitle
   - Added notes indicator (*) in monster list
   - Updated `MonsterStatsPanel.vue` to show display_name and DM notes section

8. **Fixed Additional References**
   - `seed/dev/module_data.rs` - Added None, None for new params
   - `campaign_archive_service.rs` - Updated `ModuleMonsterData` struct and From impl
   - `module_frontmatter_service.rs` - Added None, None for new fields

**Build Status:**
- Rust crates compile successfully
- TypeScript types check successfully
- All acceptance criteria met

### Session 2 - 2026-01-19

**Added UI editing and reactive data patterns:**

1. **Monster Edit Modal** (`ModulesTab.vue`)
   - Added edit button (pencil icon) to monster list items
   - Created modal form for editing display_name and notes
   - Connected to `update_module_monster` Tauri command
   - Monster list auto-refreshes after edits

2. **Event Bus for Reactive Data** (new files)
   - Created `src/shared/utils/dataEvents.ts` - type-safe event bus
   - Created `src/shared/composables/useDataEvents.ts` - Vue composable wrapper
   - Applied event pattern across entire application:
     - DocumentService, ModuleService emit events on mutations
     - Campaign store, Character store emit events
     - NPC/Map components emit events after add/remove
   - Composables listen and auto-refresh (useModuleMonsters, useModuleMaps, ModuleNPCs)

3. **Committed and Tagged**
   - Commit: `2483933` - feat: Add monster customization and reactive data event bus
   - Tag: `v0.2.4` created

**All acceptance criteria complete:**
- Backend: migration, models, services, MCP tools, Tauri commands
- Frontend: display_name shown with original as subtitle, notes indicator, edit modal
- Reactive updates: changes appear immediately without page navigation
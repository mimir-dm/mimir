---
id: gap-analysis-homebrew-system
level: task
title: "Gap analysis: homebrew system documentation (items, monsters, spells)"
short_code: "MIMIR-T-0584"
created_at: 2026-03-11T23:13:28.922783+00:00
updated_at: 2026-03-13T12:48:31.428821+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Gap analysis: homebrew system documentation (items, monsters, spells)

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

The homebrew system (items, monsters, spells) is a major feature with zero user-facing documentation. Analyze the feature surface area and produce a documentation plan specifying what pages need to be written, where they should live in the docs structure, and what screenshots are needed.

## Scope

Analyze the homebrew system across three domains — items, monsters, and spells — to determine what documentation is needed.

### Systems to Analyze
- **Homebrew Items**: `HomebrewService` in mimir-core, UI components for create/edit/list, integration with character inventory
- **Homebrew Monsters**: `HomebrewMonsterService`, VTT stat block integration, module monster list integration
- **Homebrew Spells**: `HomebrewSpellService`, character spell assignment, spell slot tracking

### Code to Review
- `crates/mimir-core/src/services/homebrew*.rs` — service layer capabilities
- `crates/mimir/frontend/src/` — homebrew UI components
- `crates/mimir-mcp/src/tools/homebrew*.rs` — MCP tool surface area
- Tauri commands for homebrew operations

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Map the complete homebrew item workflow: create → edit → assign to character → view in inventory
- [ ] Map the complete homebrew monster workflow: create → edit → add to module → place on map as token
- [ ] Map the complete homebrew spell workflow: create → edit → assign to character → view in spell list
- [ ] For each workflow, specify what how-to guide pages are needed and their proposed titles
- [ ] Propose where in the SUMMARY.md structure new homebrew docs should live (likely `how-to/homebrew/`)
- [ ] Identify all UI screens involved and flag which need screenshots — describe what each screenshot should show
- [ ] Check whether homebrew features need reference page additions (glossary terms, file format notes)
- [ ] Produce a documentation plan as the task output

## Expected Output

A documentation plan listing:
1. New pages needed (with proposed filenames and SUMMARY.md placement)
2. Existing pages that need homebrew mentions added
3. Screenshots needed (with description of what to capture)
4. Glossary terms to add

## Status Updates

### Gap Analysis Completed 2026-03-13

---

## System Surface Area

The homebrew system spans 3 entity types (items, monsters, spells) across all layers:
- **Backend:** HomebrewService with full CRUD + clone-from-catalog for all 3 types
- **Tauri commands:** 15+ commands (list/get/create/update/delete × 3 types)
- **MCP tools:** 15 tools across 3 tool modules (homebrew, homebrew_monster, homebrew_spell)
- **Frontend:** HomebrewTab with 3 sub-tabs, factory-pattern TypeScript services, stat block renderers
- **Integration:** Character inventory (items show "HB" badge), module monsters (homebrew monsters searchable alongside catalog)
- **Safety:** Delete operations check for references in character inventories and module monster lists before proceeding

## Documentation Plan

### New Pages Needed

#### 1. `docs/src/how-to/homebrew/README.md` — Homebrew Overview
- What homebrew content is and why you'd create it
- Overview of the 3 types: items, monsters, spells
- Where to find the Homebrew tab (5th campaign dashboard tab)
- Link to individual how-to pages

#### 2. `docs/src/how-to/homebrew/create-item.md` — Create Homebrew Items
- Navigate to Homebrew tab → Items sub-tab
- Two creation paths: from scratch vs clone-from-catalog
- Clone workflow: search catalog → select → modify → save
- Form fields: name, item_type, rarity, data (JSON or structured form for weapons/armor)
- Editing existing items
- Deleting items (with inventory reference warning)

#### 3. `docs/src/how-to/homebrew/create-monster.md` — Create Homebrew Monsters
- Navigate to Homebrew tab → Monsters sub-tab
- Clone-from-catalog workflow (search by name/CR/type)
- JSON data editing with stat block preview
- Adding homebrew monsters to module encounter lists
- Deleting monsters (with module reference warning)

#### 4. `docs/src/how-to/homebrew/create-spell.md` — Create Homebrew Spells
- Navigate to Homebrew tab → Spells sub-tab
- Clone-from-catalog workflow
- JSON data editing with spell stat block preview
- Assigning homebrew spells to characters
- Deleting spells

#### 5. `docs/src/explanation/homebrew-system.md` — Homebrew Concepts (Explanation)
- Homebrew vs catalog content: what's the difference?
- Campaign-scoped: homebrew content belongs to a specific campaign
- Clone-from-catalog: start from an existing item and modify
- Integration points: how homebrew flows into character sheets and modules
- Data format: JSON blobs following 5etools schema conventions

### SUMMARY.md Placement

```markdown
- [How-To Guides](./how-to/README.md)
  - [Campaigns](./how-to/campaigns/README.md)
    - ...existing pages...
  - [Homebrew](./how-to/homebrew/README.md)        ← NEW SECTION
    - [Create Items](./how-to/homebrew/create-item.md)
    - [Create Monsters](./how-to/homebrew/create-monster.md)
    - [Create Spells](./how-to/homebrew/create-spell.md)
  - [Characters](./how-to/characters/README.md)
    - ...existing pages...
- [Explanation](./explanation/README.md)
  - ...existing pages...
  - [Homebrew System](./explanation/homebrew-system.md)  ← NEW PAGE
```

### Existing Pages Needing Homebrew Mentions

| Page | What to add |
|------|-------------|
| `tutorials/01-first-campaign.md` | Mention 5th Homebrew tab in dashboard overview |
| `how-to/modules/add-monsters.md` | Note that homebrew monsters appear in search alongside catalog |
| `how-to/characters/create-pc.md` | Mention Equipment tab can hold homebrew items |
| `reference/ui/campaign-dashboard.md` | Add Homebrew tab section |
| `reference/ui/token-setup-modal.md` | Note homebrew monsters in Module Monsters quick-select |
| `reference/glossary.md` | Add "Homebrew" term definition |
| `explanation/two-board-system.md` | Add Homebrew tab to Campaign Board description |

### Screenshots Needed

| Screenshot | What to capture |
|-----------|----------------|
| `homebrew-tab.png` | Homebrew tab showing items sub-tab with item list |
| `homebrew-clone.png` | Clone-from-catalog modal with search results |
| `homebrew-item-edit.png` | Item edit form showing structured fields |
| `homebrew-monster.png` | Monster sub-tab with stat block preview |
| `homebrew-spell.png` | Spell sub-tab with spell stat block preview |
| `homebrew-delete-warning.png` | Delete confirmation showing affected characters/modules |
| `homebrew-badge.png` | Character inventory showing "HB" badge on homebrew items |

### Glossary Terms to Add

- **Homebrew** — Custom content (items, monsters, spells) created by the DM within a campaign, stored alongside catalog content
- **Clone from Catalog** — Feature to create homebrew content by copying and modifying an existing catalog entry
- **Catalog** — The built-in D&D 5e reference data (from 5etools), read-only
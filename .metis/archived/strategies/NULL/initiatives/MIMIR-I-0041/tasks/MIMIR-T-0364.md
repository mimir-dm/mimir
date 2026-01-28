---
id: v0-5-pdf-and-print-export-design
level: task
title: "v0.5 PDF and Print Export Design"
short_code: "MIMIR-T-0364"
created_at: 2026-01-20T01:22:50.338295+00:00
updated_at: 2026-01-28T03:50:54.869077+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 PDF and Print Export Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Design the PDF generation and print export system using Typst. Define available export types, configurable sections, and the ExportService API.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] All exportable content types defined
- [ ] Typst template structure specified
- [ ] ExportService trait with all methods
- [ ] Print section options documented
- [ ] Output format specifications

## Technology

**Typst** - Modern typesetting system (like LaTeX but simpler)
- Rust-native via `typst` crate
- Templates are `.typ` files with markup + scripting
- Renders directly to PDF

## Exportable Content Types

### Character Exports

| Export Type | Description | Sections |
|-------------|-------------|----------|
| **Character Sheet** | Full 5e character sheet | Stats, abilities, skills, equipment, spells |
| **Character Card** | Compact reference card | Name, class, key stats, HP/AC |
| **NPC Sheet** | Simplified for NPCs | Stats, role, notes, key abilities |

### Monster Exports

| Export Type | Description |
|-------------|-------------|
| **Stat Block** | Full 5e stat block format |
| **Monster Card** | Compact card for quick reference |
| **Encounter Sheet** | Multiple monsters grouped by encounter_tag |

### Module Exports

| Export Type | Description | Configurable Sections |
|-------------|-------------|----------------------|
| **Module PDF** | Complete module package | Documents, monsters, items, NPCs, maps |

Configurable sections for module export:
- [ ] Documents (all markdown content)
- [ ] Monster Stat Blocks
- [ ] Item Cards
- [ ] NPC Sheets
- [ ] Trap Cards
- [ ] Map Previews
- [ ] Play Tiles (1":5ft scale)
- [ ] Token Cutouts

### Map Exports

| Export Type | Description |
|-------------|-------------|
| **Map Preview** | Single page, fit to page |
| **Play Tiles** | Multi-page at 1":5ft scale for tabletop |
| **Token Sheet** | Cutout sheet for physical tokens |

Map export options:
- Show/hide grid overlay
- Show/hide LOS walls
- Show/hide token positions
- Include fog state or show full map

### Spell/Item Cards

| Export Type | Description |
|-------------|-------------|
| **Spell Cards** | Printable spell cards (multiple per page) |
| **Item Cards** | Printable item cards |
| **Trap Cards** | Printable trap/hazard cards |

## ExportService Trait

```rust
#[async_trait]
pub trait ExportService: Send + Sync {
    // Character exports
    async fn character_sheet(&self, id: i64) -> Result<Vec<u8>>;
    async fn character_card(&self, id: i64) -> Result<Vec<u8>>;
    async fn npc_sheet(&self, id: i64) -> Result<Vec<u8>>;
    
    // Monster exports
    async fn monster_stat_block(&self, name: &str, source: &str) -> Result<Vec<u8>>;
    async fn monster_card(&self, name: &str, source: &str) -> Result<Vec<u8>>;
    async fn encounter_sheet(&self, module_id: i64, encounter_tag: &str) -> Result<Vec<u8>>;
    
    // Module exports
    async fn module_pdf(&self, id: i64, options: ModuleExportOptions) -> Result<Vec<u8>>;
    
    // Map exports
    async fn map_preview(&self, id: i64, options: MapExportOptions) -> Result<Vec<u8>>;
    async fn map_play_tiles(&self, id: i64, options: MapExportOptions) -> Result<Vec<u8>>;
    async fn token_sheet(&self, map_id: i64) -> Result<Vec<u8>>;
    
    // Card exports
    async fn spell_cards(&self, spell_names: &[(String, String)]) -> Result<Vec<u8>>;
    async fn item_cards(&self, item_names: &[(String, String)]) -> Result<Vec<u8>>;
    async fn trap_cards(&self, trap_names: &[(String, String)]) -> Result<Vec<u8>>;
}

pub struct ModuleExportOptions {
    pub include_documents: bool,
    pub include_monsters: bool,
    pub include_items: bool,
    pub include_npcs: bool,
    pub include_traps: bool,
    pub include_map_previews: bool,
    pub include_play_tiles: bool,
    pub include_token_sheets: bool,
}

pub struct MapExportOptions {
    pub show_grid: bool,
    pub show_los_walls: bool,
    pub show_tokens: bool,
    pub show_fog: bool,  // false = show full map
    pub scale: MapScale,
}

pub enum MapScale {
    FitToPage,
    OneInchFiveFeet,  // 1":5ft for tabletop play
}
```

## Typst Template Structure

```
templates/
├── character/
│   ├── sheet.typ        # Full character sheet
│   ├── card.typ         # Compact card
│   └── npc.typ          # NPC-focused sheet
├── monster/
│   ├── statblock.typ    # Standard stat block
│   ├── card.typ         # Compact card
│   └── encounter.typ    # Multi-monster encounter
├── module/
│   ├── main.typ         # Module wrapper
│   ├── document.typ     # Markdown content section
│   └── sections.typ     # Reusable section components
├── map/
│   ├── preview.typ      # Single page map
│   ├── tiles.typ        # Multi-page play tiles
│   └── tokens.typ       # Token cutout sheet
├── cards/
│   ├── spell.typ        # Spell card
│   ├── item.typ         # Item card
│   └── trap.typ         # Trap card
└── common/
    ├── styles.typ       # Shared styles
    ├── fonts.typ        # Font configuration
    └── dnd-theme.typ    # D&D visual theme
```

## Data Flow

```
Service Layer                    Export Service                  Typst
     │                                │                            │
     │  get_character(id)             │                            │
     ├───────────────────────────────>│                            │
     │                                │  load template              │
     │                                │  inject data                │
     │                                ├───────────────────────────>│
     │                                │                   render    │
     │                                │<───────────────────────────┤
     │          PDF bytes             │                            │
     │<───────────────────────────────┤                            │
```

## UI Integration

Export dialogs in the UI:

### Character Export Dialog
- Export type: Sheet / Card / NPC
- Preview thumbnail
- Download button

### Module Export Dialog
- Checklist of sections to include
- Preview of page count estimate
- Download button

### Map Export Dialog
- Export type: Preview / Play Tiles / Tokens
- Grid/LOS/Token toggles
- Preview
- Download button

## Dependencies

- Depends on: [[MIMIR-T-0357]] Database Schema (data models)
- Depends on: [[MIMIR-T-0358]] Service Layer (data access)

## Progress

*To be updated during implementation*
---
id: pdf-export-system-rework
level: initiative
title: "PDF Export System Rework"
short_code: "MIMIR-I-0027"
created_at: 2025-12-25T02:41:15.291813+00:00
updated_at: 2025-12-25T15:48:03.410058+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/ready"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: pdf-export-system-rework
---

# PDF Export System Rework Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

The PDF export system is a core feature allowing DMs to generate printable materials from their campaign data. The current implementation uses Typst (a modern typesetting system) to compile templates with JSON data into PDFs. While the architecture is sound, the system has become unreliable and needs a comprehensive rework.

### Current State

**Technology Stack:**
- `typst` 0.12 for document compilation
- `typst-pdf` 0.12 for PDF generation
- `fontdb` for system font loading
- Tauri commands exposing functionality to Vue frontend
- 21 Typst templates organized by category

**Supported Export Types:**
- Character sheets (full, summary, with-spells variants)
- Spell cards and lists (individual, multi-up, list format)
- Monster stat blocks (single, cards, encounter sheets)
- Session materials (NPC cards, prep sheets, handouts)
- Campaign documents (single and combined exports)
- Map exports with grid overlay and tokens

**Architecture:**
```
Vue UI → PrintService.ts → Tauri Commands → PrintService (Rust)
                                               ↓
                              MimirTypstWorld (fonts, file resolution)
                                               ↓
                              typst::compile() → typst_pdf::pdf()
```

### Known Issues

1. **Template Reliability**
   - Recent commits show ongoing debugging of PDF failures
   - Some templates may not handle all data variations gracefully
   - Missing test coverage makes regressions easy to introduce

2. **Incomplete Specifications**
   - No documented data schema for each template
   - Unclear which fields are required vs optional
   - Behavior for edge cases not defined

3. **Inconsistent Styling**
   - Templates may have diverged from shared styles
   - Font usage not standardized across templates
   - Visual design quality varies by template

4. **No Test Coverage**
   - No automated tests for template compilation
   - No sample data fixtures for testing
   - Regressions discovered only in production use

### Key Files

| Component | Location | Lines |
|-----------|----------|-------|
| PrintService (Rust) | `crates/mimir-dm-print/src/service.rs` | ~210 |
| MimirTypstWorld | `crates/mimir-dm-print/src/world.rs` | ~235 |
| Tauri Commands | `crates/mimir-dm/src/commands/print/mod.rs` | **~1600** |
| Frontend Service | `crates/mimir-dm/frontend/src/services/PrintService.ts` | ~380 |
| PDF Preview Modal | `crates/mimir-dm/frontend/src/components/print/PdfPreviewModal.vue` | ~100 |
| Templates | `crates/mimir-dm-print/templates/` | 21 files |

## Architecture Review

### Good Pieces (Keep)

| Component | Why Keep | Notes |
|-----------|----------|-------|
| `PrintService` | Clean abstraction | `render_to_pdf()`, `list_templates()`, `save_pdf()` |
| `MimirTypstWorld` | Proper Typst World impl | Font loading, JSON injection, file resolution |
| `_shared/styles.typ` | Good design system | Colors, typography, spacing constants |
| `_shared/components.typ` | Reusable UI components | `ability-scores`, `info-box`, `labeled-value` |
| `strip_5etools_tags()` | Works well | Regex-based tag conversion for monster data |
| Template test fixtures | Good coverage | Sample data in `service.rs` tests |

### Problems (Rewrite)

| Component | Problem | Solution |
|-----------|---------|----------|
| `print/mod.rs` | **1600 lines, too many responsibilities** | Split into focused modules |
| Data gathering | Each command does own DB queries (~230 lines for character) | Create `PrintDataService` to prepare data |
| Template path resolution | Uses `CARGO_MANIFEST_DIR`, won't work in production | Proper Tauri resource bundling |
| No map printing | Missing entirely | Add map templates and image handling |
| No print options UI | Frontend has no template picker | Add options dialog per print path |
| Duplicated patterns | Each export command repeats connection handling | Extract common patterns |

### Proposed New Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND                                 │
├─────────────────────────────────────────────────────────────────┤
│  PrintOptionsDialog.vue    - Template picker, options per path  │
│  PdfPreviewModal.vue       - Display, save, print (keep)        │
│  PrintService.ts           - Simplified, path-based methods     │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Tauri invoke
┌──────────────────────────┴──────────────────────────────────────┐
│                      TAURI COMMANDS                              │
│                   (thin layer, ~200 lines)                       │
├─────────────────────────────────────────────────────────────────┤
│  print_document(doc_id, template)                               │
│  print_character(char_id, template)                             │
│  print_map(map_id, options)                                     │
│  print_bulk(selection)                                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────────┐
│                    PRINT DATA SERVICE                            │
│               (mimir-dm-core, new module)                        │
├─────────────────────────────────────────────────────────────────┤
│  prepare_character_data(id) -> CharacterPrintData               │
│  prepare_document_data(id) -> DocumentPrintData                 │
│  prepare_map_data(id, options) -> MapPrintData                  │
│  prepare_bulk_data(selection) -> BulkPrintData                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────────┐
│                      PRINT SERVICE                               │
│                   (mimir-dm-print, keep)                         │
├─────────────────────────────────────────────────────────────────┤
│  render_to_pdf(template, data) -> Vec<u8>                       │
│  MimirTypstWorld (keep)                                         │
│  Templates (refine)                                              │
└─────────────────────────────────────────────────────────────────┘
```

### Key Changes

1. **Split `print/mod.rs` into:**
   - `mod.rs` - Just command registration
   - `document.rs` - Single doc printing
   - `character.rs` - Character sheet printing  
   - `map.rs` - Map printing (new)
   - `bulk.rs` - Bulk export

2. **New `PrintDataService` in `mimir-dm-core`:**
   - All data gathering logic moves here
   - Returns typed structs, not raw JSON
   - Testable without Tauri

3. **Template bundling:**
   - Use Tauri's `tauri.conf.json` resources
   - `PrintService::new()` takes resource path from app handle

4. **Map printing (new):**
   - `templates/map/full.typ` - Map with overlays
   - `templates/map/tiled.typ` - Multi-page tiled
   - `templates/map/cutouts.typ` - Token standees
   - Image handling in `MimirTypstWorld`

## Goals & Non-Goals

**Goals:**
- Clearly defined behavior for each PDF export type
- Well-designed, consistent templates across all export types
- Comprehensive test coverage with sample data fixtures
- Reliable generation that fails gracefully with clear errors

**Non-Goals:**
- Architectural overhaul of the Typst integration
- Custom template editor UI
- Cloud-based PDF generation
- Font bundling (system fonts are acceptable)

## Print Entry Points

The PDF export system has three distinct paths users take to generate PDFs. Each has different contexts, selection capabilities, and output types.

---

### Path 1: Individual Document Print

**Entry Point:** Print button in document sidebar (when viewing a single document)

**Context:** User is viewing/editing a single document within a campaign/module and wants to print just that item.

**Constraint:** Printing is only available for items linked to a campaign. The catalog/reference browser does not have print functionality - users must add items to a campaign first.

**Applicable Document Types:**
| Document Type | Template Used | Notes |
|---------------|---------------|-------|
| Campaign note/document | `campaign/document` | Markdown → PDF |
| Module document | `campaign/document` | Same template |
| Session notes | `campaign/document` | Play notes from session |
| Module monster | `monsters/stat-block` | Monster added to module |
| Module NPC | `session/npc-card` | NPC created in module |
| Handout | `session/handout` | Player-facing prop |

**UI Flow:**
1. User opens document in editor/viewer (within campaign context)
2. Clicks "Print" button in sidebar/toolbar
3. PDF generates immediately (no options needed for most types)
4. Preview modal opens with Save/Print/Close

**Template Selection:**
Show template picker when multiple formats available:

| Item Type | Options | Default |
|-----------|---------|---------|
| Monster | Stat Block, Card | Stat Block |
| NPC | NPC Card, Full Character Sheet | NPC Card |

---

### Path 2: Bulk Export from Views

**Entry Point:** Export button in major view components (Module view, Campaign view)

**Context:** User wants to export multiple related documents as a bundle. Used for:
- Preparing session materials (all NPCs, encounters for a module)
- Archiving a completed campaign
- Onboarding materials for new campaign

**Available from Module View:**
| Selection | Template | Output |
|-----------|----------|--------|
| All module documents | `campaign/combined` | Single PDF with TOC |
| All module NPCs | `session/npc-cards-multiup` | Multi-up NPC cards |
| All module monsters | `monsters/cards-multiup` | Multi-up monster cards |
| Module encounter sheet | `monsters/encounter` | Grouped stat blocks |
| Module play notes | `campaign/document` | Session notes document |

**Available from Campaign View:**
| Selection | Template | Output |
|-----------|----------|--------|
| All campaign documents | `campaign/combined` | Combined narrative docs |
| Full campaign export | *Multiple* | See "Full Campaign Export" below |

**UI Flow:**
1. User navigates to Module or Campaign view
2. Clicks "Export" dropdown in view header
3. Selects export type from menu:
   - "Export All Documents"
   - "Export NPC Cards"
   - "Export Monster Cards"
   - "Full Campaign Export..." (Campaign view only)
4. For simple exports: generates immediately, shows preview
5. For Full Campaign Export: opens selection dialog (see below)

---

### Path 3: Map Print

**Entry Point:** Print button in Map Viewer (within module/campaign)

**Context:** DM wants to print a map for physical tabletop play, optionally with grid and token positions.

**Print Options:**

| Option | Description | Use Case |
|--------|-------------|----------|
| Map Only | Clean map image, no overlays | Scenic handout, player map |
| Map + Grid | Map with grid overlay | Physical play with own minis |
| Map + Grid + Tokens | Grid overlay with tokens shown on map | Reference for DM setup |
| Map + Grid + Token Cutouts | Grid map + separate page of token standees | Full physical play kit |

**UI Flow:**
1. User opens map in Map Viewer
2. Clicks "Print" button in map toolbar
3. Print options dialog appears:
   ```
   ┌─────────────────────────────────────┐
   │  Print Map                          │
   ├─────────────────────────────────────┤
   │                                     │
   │  Include:                           │
   │  ☑ Grid Overlay                     │
   │  ☐ Token Positions                  │
   │  ☐ Token Cutouts (separate page)    │
   │                                     │
   │  Scale:                             │
   │  ○ Fit to page                      │
   │  ● 1" = 5ft (standard grid)         │
   │  ○ Custom: [___] inches per square  │
   │                                     │
   │  Pages: 1 (Letter) or 4 (tiled)     │
   │                                     │
   ├─────────────────────────────────────┤
   │              [Cancel] [Print]       │
   └─────────────────────────────────────┘
   ```
4. PDF generates with selected options
5. Preview modal opens

**Token Cutouts Page:**
When "Token Cutouts" is selected, append a page with:
- Each token as a circular/square image at grid scale
- Token name beneath each
- Fold lines for paper standees
- Arranged in grid for easy cutting

**Scale Options:**
| Option | Behavior |
|--------|----------|
| Fit to page | Scales map to fit single page (may distort grid) |
| 1" = 5ft | Standard D&D grid, may tile across multiple pages |
| Custom | User-specified scale for specific table setups |

**Multi-Page Tiling:**
For large maps at true scale:
- Split across multiple letter/A4 pages
- Each page shows adjacent piece numbers in margins for assembly
- Blank margins indicate map edges

Example margin labels for a 2x3 tiled map (page 3):
```
         ┌─────────────────────┐
         │         2           │  ← "glue to page 2"
         │                     │
    ─────┼─────────────────────┼─────
         │                     │
  blank  │      Page 3         │  4   ← "glue to page 4"
 (edge)  │       of 6          │
         │                     │
    ─────┼─────────────────────┼─────
         │                     │
         │         6           │  ← "glue to page 6"
         └─────────────────────┘
```

**Assembly Diagram:**
First page includes a thumbnail assembly guide showing the grid layout:
```
┌───┬───┐
│ 1 │ 2 │
├───┼───┤
│ 3 │ 4 │
├───┼───┤
│ 5 │ 6 │
└───┴───┘
```

---

### Path 4: Character Sheet Print

**Entry Point:** Print button in Character Editor

**Context:** User is editing a character (PC or NPC) and wants a printable sheet.

**Template Options:**
| Option | Template | Description |
|--------|----------|-------------|
| Full Sheet | `character/sheet` | Multi-page complete reference |
| Summary | `character/summary` | Single-page quick reference |
| Sheet + Spell Cards | `character/sheet-with-spells` | Full sheet followed by spell cards |

**UI Flow:**
1. User opens character in editor
2. Clicks "Print" button in character header
3. Template selection dropdown appears:
   - "Full Character Sheet"
   - "Quick Reference (1 page)"
   - "Character Sheet with Spell Cards" (if spellcaster)
4. PDF generates with selected template
5. Preview modal opens

**Spellcaster Detection:**
- Show "with Spell Cards" option only if character has spellcasting
- Include all prepared/known spells + full class spell list for reference

---

## Full Campaign Export (Deep Dive)

This is the most complex export path, used for comprehensive campaign archival or onboarding.

**Entry Point:** Campaign View → Export → "Full Campaign Export..."

**Use Cases:**
1. **New Campaign Onboarding:** Player wants all materials to prep for joining
2. **Campaign Archival:** DM wants complete physical record of completed campaign
3. **Session Prep Bundle:** DM wants everything for upcoming session

**Selection Dialog:**

```
┌─────────────────────────────────────────────────────────┐
│  Full Campaign Export                                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ☑ Campaign Documents                                    │
│    ├─ ☑ Session Notes (12 documents)                    │
│    ├─ ☑ Lore & World (8 documents)                      │
│    └─ ☑ Player Handouts (3 documents)                   │
│                                                          │
│  ☑ Characters                                            │
│    ├─ ☑ Player Characters (4)                           │
│    │     Template: [Full Sheet ▼]                       │
│    └─ ☐ NPCs (23)                                       │
│          Template: [NPC Cards ▼]                        │
│                                                          │
│  ☑ Modules                                               │
│    ├─ ☑ The Lost Mine (active)                          │
│    │     ├─ ☑ Documents                                 │
│    │     ├─ ☑ Monsters                                  │
│    │     └─ ☑ NPCs                                      │
│    └─ ☐ Dragon Heist (archived)                         │
│                                                          │
│  ☐ Reference Materials                                   │
│    ├─ ☐ Spell Cards (for all PCs)                       │
│    └─ ☐ Monster Quick Reference                         │
│                                                          │
├─────────────────────────────────────────────────────────┤
│  Estimated: ~45 pages                    [Cancel] [Export]│
└─────────────────────────────────────────────────────────┘
```

**Output:**
Single combined PDF with TOC, page numbers, and section dividers.

**Section Order:**
1. Title page (Campaign name, export date)
2. Table of Contents
3. Player Characters (full sheets)
4. Campaign Documents (by category)
5. Module Materials (per module)
6. Reference Materials (spell cards, etc.)

**Progress Indication:**
- Show progress bar for large exports
- "Generating page X of ~Y..."
- Allow cancellation

---

## Use Cases

### UC-1: Generate Character Sheet
- **Actor**: DM preparing session materials
- **Scenario**: 
  1. Open character detail view
  2. Click "Export PDF" button
  3. Select template variant (full/summary/with-spells)
  4. Preview generated PDF
  5. Save or print PDF
- **Expected Outcome**: Properly formatted PDF with all character data

### UC-2: Print Spell Cards
- **Actor**: Player or DM wanting physical spell reference
- **Scenario**:
  1. Select spells from catalog or character spellbook
  2. Choose card layout (single/multi-up for cutting)
  3. Generate PDF with selected spells
  4. Print on cardstock
- **Expected Outcome**: Printable cards at correct dimensions

### UC-3: Export Campaign Documents
- **Actor**: DM archiving or printing campaign notes
- **Scenario**:
  1. Navigate to campaign view
  2. Select "Export All Documents"
  3. Choose combined or separate PDFs
  4. Wait for generation with progress indicator
  5. Save combined PDF
- **Expected Outcome**: Single PDF with table of contents

### UC-4: Create NPC Handout
- **Actor**: DM preparing materials for session
- **Scenario**:
  1. Select NPC from module
  2. Choose "Generate Handout"
  3. Select handout style (aged, formal, etc.)
  4. Customize visible fields
  5. Save/print for players
- **Expected Outcome**: Stylized handout hiding DM-only info

### UC-5: Generate Monster Encounter Sheet
- **Actor**: DM preparing combat encounter
- **Scenario**:
  1. Select monsters for encounter
  2. Choose encounter sheet template
  3. Add DM notes for the encounter
  4. Generate single-page reference
  5. Print for session use
- **Expected Outcome**: Consolidated stat blocks with notes

## Feature Specifications

### Character Sheet Exports

| Template | Purpose | Key Data |
|----------|---------|----------|
| `character/sheet` | Full character sheet (multi-page) | All character data, abilities, features, equipment |
| `character/summary` | Quick reference (1 page) | Core stats, AC, HP, key abilities |
| `character/sheet-with-spells` | Full sheet + spell cards | Character data + prepared/known spells |

**Behavior:**
- Should handle characters at any level (1-20)
- Multiclass characters show combined features
- Equipment includes attunement status
- Spellcasters show spell slots and prepared spells
- NPCs use same templates with NPC-specific fields (CR, legendary actions)

### Spell Exports

| Template | Purpose | Key Data |
|----------|---------|----------|
| `spells/card` | Single spell card | One spell with full details |
| `spells/cards-multiup` | Printable card sheet | Multiple spells, cut lines for printing |
| `spells/list` | Compact spell list | Spell names, levels, schools, brief descriptions |

**Behavior:**
- Cards sized for standard card sleeves (2.5" x 3.5")
- Multi-up layout: 9 cards per letter page (3x3)
- Long spell descriptions should wrap gracefully
- Ritual/concentration tags clearly visible
- Components (V, S, M) with material costs shown

### Monster Exports

| Template | Purpose | Key Data |
|----------|---------|----------|
| `monsters/stat-block` | Full stat block | Complete monster data in PHB style |
| `monsters/card` | Quick reference card | Core stats, actions, key abilities |
| `monsters/cards-multiup` | Printable card sheet | Multiple monsters for encounter |
| `monsters/encounter` | Encounter reference | Multiple monsters with DM notes |

**Behavior:**
- Stat blocks follow 5e SRD formatting conventions
- Legendary actions and lair actions in separate sections
- CR and XP displayed prominently
- Multi-attack formatted as nested actions
- Spellcasting shows spell list with slots

### Session Materials

| Template | Purpose | Key Data |
|----------|---------|----------|
| `session/prep` | DM prep sheet | NPCs, locations, encounters, hooks, secrets |
| `session/npc-card` | Single NPC card | Name, role, personality, secrets |
| `session/npc-cards-multiup` | Multiple NPC cards | Batch print for session |
| `session/handout` | Player handout | Styled document for in-game props |

**Behavior:**
- Prep sheet fits on 1-2 pages for at-table use
- NPC cards hide DM-only fields (secrets) by default
- Handouts support multiple visual styles (aged, formal, magical)
- Handout body supports markdown formatting

### Campaign Documents

| Template | Purpose | Key Data |
|----------|---------|----------|
| `campaign/document` | Single document | Markdown content with frontmatter |
| `campaign/combined` | All campaign docs | Multiple documents with TOC |

**Behavior:**
- Documents preserve markdown formatting (headers, lists, emphasis)
- Combined export generates table of contents
- Page breaks between documents in combined mode
- Frontmatter (title, date, category) shown in header

### Map Exports

**Behavior:**
- Export map image with grid overlay
- Token positions shown on map
- Grid scale and dimensions in footer
- Optional: fog of war state (revealed areas only)

## Template Design Standards

### Visual Consistency
- Use shared `_shared/styles.typ` for colors, fonts, spacing
- Headers: Dark background with white text
- Body text: 10pt minimum for readability
- Tables: Alternating row colors for scannability

### Layout Principles
- Character sheets: Dense but organized, use columns
- Cards: High contrast, large text for key info
- Prep sheets: Scannable sections, bullet points
- Handouts: Thematic, immersive styling

### Required Fonts
Templates should use fonts commonly available:
- **Body**: System sans-serif (Inter, Helvetica, Arial)
- **Headers**: System sans-serif bold
- **Monospace**: JetBrains Mono, Consolas, or system mono

## Testing Strategy

### Test Fixtures

Create JSON fixtures in `crates/mimir-dm-print/tests/fixtures/`:

```
fixtures/
├── character/
│   ├── level-1-fighter.json
│   ├── level-10-multiclass.json
│   ├── level-20-wizard.json
│   └── npc-with-legendary.json
├── spells/
│   ├── cantrip.json
│   ├── ritual-spell.json
│   ├── long-description.json
│   └── spell-list-by-class.json
├── monsters/
│   ├── simple-beast.json
│   ├── spellcaster.json
│   ├── legendary-creature.json
│   └── encounter-group.json
├── session/
│   ├── npc-minimal.json
│   ├── npc-full.json
│   ├── prep-sheet.json
│   └── handout-aged.json
└── campaign/
    ├── simple-document.json
    └── multi-document.json
```

### Unit Tests

For each template:
1. **Minimal data test** - Only required fields, should not error
2. **Full data test** - All fields populated, verify output
3. **Edge cases** - Empty arrays, Unicode text, very long content

### Integration Tests

1. Generate PDF from each template with fixture data
2. Verify PDF is valid (non-zero size, parseable)
3. Check page count matches expectations

### Regression Prevention

- Store "golden" PDFs for visual comparison
- Run diff on CI when templates change
- Manual review required for intentional visual changes

## Implementation Plan

### Phase 1: Architecture Refactor

**Goal:** Clean separation of concerns, testable code

1. **Create `PrintDataService` in `mimir-dm-core`**
   - `CharacterPrintData` struct with all needed fields
   - `DocumentPrintData` struct for campaign docs
   - `MapPrintData` struct with grid/token options
   - `BulkPrintSelection` for campaign export
   - Move data gathering logic from commands here
   - Add unit tests for each prepare method

2. **Split `print/mod.rs` into modules**
   - `document.rs` - `print_document` command
   - `character.rs` - `print_character` command
   - `map.rs` - `print_map` command (new)
   - `bulk.rs` - `print_bulk` command
   - Keep commands thin: validate, call service, return

3. **Fix template bundling**
   - Add templates to `tauri.conf.json` resources
   - Update `PrintService::new()` to accept resource path
   - Test in release build

### Phase 2: Map Printing

**Goal:** Full physical play support

4. **Add map templates**
   - `map/full.typ` - Single page with overlays
   - `map/tiled.typ` - Multi-page with assembly guide
   - `map/cutouts.typ` - Token standee sheet

5. **Extend `MimirTypstWorld` for images**
   - Handle map image loading
   - Handle token image loading
   - Scale calculations for grid sizing

6. **Add map print options dialog**
   - Grid overlay toggle
   - Token positions toggle
   - Token cutouts toggle
   - Scale selector (fit/1"=5ft/custom)

### Phase 3: Frontend Polish

**Goal:** Complete print UX for all paths

7. **Create `PrintOptionsDialog.vue`**
   - Template picker for items with multiple formats
   - Preview of selected template
   - Per-path options (map scale, etc.)

8. **Simplify `PrintService.ts`**
   - One method per print path
   - Options passed as typed objects
   - Remove legacy code paths

9. **Add print buttons to all views**
   - Document sidebar print button
   - Character editor print button
   - Map viewer print button
   - Module/Campaign export menus

### Phase 4: Templates & Testing

**Goal:** Reliable, well-tested templates

10. **Create test fixtures directory**
    - `fixtures/character/` - level 1, multiclass, NPC
    - `fixtures/monster/` - simple, legendary, spellcaster
    - `fixtures/map/` - small, large tiled, with tokens

11. **Add template compilation tests**
    - Each template compiles with minimal data
    - Each template compiles with full data
    - Edge cases: empty arrays, Unicode, long text

12. **Template refinement**
    - Audit against feature specifications
    - Fix any layout/styling issues
    - Ensure consistent use of `_shared/`

### Phase 5: Documentation

13. **Update user docs**
    - Print button locations
    - Template options
    - Map printing guide

14. **Add template authoring guide**
    - Data schema per template
    - Shared component reference
    - How to customize
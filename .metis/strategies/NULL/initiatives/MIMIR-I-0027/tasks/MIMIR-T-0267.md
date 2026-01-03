---
id: implement-renderable-trait-and
level: task
title: "Implement Renderable trait and DocumentBuilder architecture"
short_code: "MIMIR-T-0267"
created_at: 2026-01-01T18:10:13.844750+00:00
updated_at: 2026-01-01T18:11:14.949085+00:00
parent: MIMIR-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0027
---

# Implement Renderable trait and DocumentBuilder architecture

Refactor `mimir-dm-print` to use a composable trait-based architecture that eliminates the current combinatorial explosion of render methods.

## Parent Initiative

[[MIMIR-I-0027]]

## Objective

Replace the current proliferation of `render_campaign_combined_with_*` methods with a composable `Renderable` trait and `DocumentBuilder` pattern that allows any combination of document types to be assembled and rendered to PDF.

## Problem Statement

Current state has methods like:
- `render_campaign_combined`
- `render_campaign_combined_with_monsters`
- `render_campaign_combined_with_monsters_and_npcs`
- `render_campaign_combined_with_all`
- `render_campaign_combined_with_all_extended`

This is unmaintainable - adding a new document type requires touching multiple methods. Testing is difficult because each combination needs separate tests.

## Proposed Architecture

### Core Trait

```rust
/// Anything that can be rendered to Typst content
pub trait Renderable {
    /// Convert to Typst markup string
    fn to_typst(&self, ctx: &RenderContext) -> Result<String>;
    
    /// Title for TOC entry (None = no TOC entry)
    fn toc_title(&self) -> Option<String> { None }
}
```

### Document Types (each implements Renderable)

| Type | Purpose | Input |
|------|---------|-------|
| `MarkdownSection` | Campaign/module documents | Parsed markdown + frontmatter |
| `MapPreview` | Single-page map overview | RenderMap + tokens |
| `TiledMapSection` | Multi-page playable map | RenderMap + tokens + grid options |
| `TokenCutoutSheet` | Printable standees | Vec<RenderToken> |
| `MonsterAppendix` | Monster stat blocks | Vec<Monster> JSON |
| `NpcAppendix` | NPC reference cards | Vec<Npc> JSON |
| `CharacterSheet` | PC/NPC character sheet | Character JSON |

### DocumentBuilder

```rust
pub struct DocumentBuilder {
    title: String,
    sections: Vec<Box<dyn Renderable>>,
    config: DocumentConfig,
}

pub struct DocumentConfig {
    pub fonts: FontConfig,
    pub include_toc: bool,
    pub page_numbers: bool,
}

impl DocumentBuilder {
    pub fn new(title: &str) -> Self;
    pub fn with_toc(self, include: bool) -> Self;
    pub fn with_fonts(self, fonts: FontConfig) -> Self;
    pub fn append<R: Renderable + 'static>(self, section: R) -> Self;
    pub fn to_pdf(&self, service: &PrintService) -> Result<Vec<u8>>;
}
```

### Usage Example

```rust
let pdf = DocumentBuilder::new("Lost Mine of Phandelver")
    .with_toc(true)
    
    // Campaign documents
    .append(MarkdownSection::from_file(&campaign_pitch)?)
    .append(MarkdownSection::from_file(&session_notes)?)
    
    // Module 1 content
    .append(MarkdownSection::from_file(&module1_overview)?)
    .append(MapPreview::new(&cragmaw_map, &tokens)?)
    .append(TiledMapSection::new(&cragmaw_map, &tokens, GridOptions::default())?)
    .append(TokenCutoutSheet::new(&tokens)?)
    
    // Reference appendices
    .append(MonsterAppendix::new(&monsters)?)
    .append(NpcAppendix::new(&npcs)?)
    
    .to_pdf(&service)?;
```

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Renderable` trait defined with `to_typst()` and optional `toc_title()`
- [ ] `DocumentBuilder` assembles sections and generates combined Typst
- [ ] `MarkdownSection` implements Renderable (replaces campaign doc rendering)
- [ ] `MapPreview` implements Renderable (single-page fit-to-page map)
- [ ] `TiledMapSection` implements Renderable (multi-page playable map)
- [ ] `TokenCutoutSheet` implements Renderable (standee cutouts)
- [ ] `MonsterAppendix` implements Renderable (monster stat blocks)
- [ ] `NpcAppendix` implements Renderable (NPC cards)
- [ ] TOC generation works when enabled
- [ ] Page numbers work across all sections
- [ ] Existing Tauri commands updated to use new architecture
- [ ] All existing integration tests pass
- [ ] New unit tests for each Renderable implementation

## Implementation Plan

### Step 1: Define Core Types
- Create `src/builder.rs` with `Renderable` trait
- Create `DocumentBuilder` struct
- Create `RenderContext` for shared state (temp dirs, fonts, etc.)

### Step 2: Implement MarkdownSection
- Extract markdown→Typst logic from current `campaign.rs`
- Implement `Renderable` for `MarkdownSection`
- Test with existing campaign document fixtures

### Step 3: Implement Map Sections
- `MapPreview` - adapt from current `render_map_with_grid`
- `TiledMapSection` - adapt from current tiled map logic
- `TokenCutoutSheet` - adapt from current cutout logic

### Step 4: Implement Reference Sections
- `MonsterAppendix` - render monster stat blocks
- `NpcAppendix` - render NPC cards

### Step 5: Update Tauri Commands
- Replace `render_campaign_combined_with_*` calls with `DocumentBuilder`
- Maintain API compatibility for frontend

### Step 6: Remove Old Code
- Delete deprecated `render_campaign_combined_*` methods
- Clean up unused helpers

## Files to Modify

| File | Changes |
|------|---------|
| `crates/mimir-dm-print/src/lib.rs` | Export new types |
| `crates/mimir-dm-print/src/builder.rs` | NEW - Renderable trait, DocumentBuilder |
| `crates/mimir-dm-print/src/sections/mod.rs` | NEW - Section types module |
| `crates/mimir-dm-print/src/sections/markdown.rs` | NEW - MarkdownSection |
| `crates/mimir-dm-print/src/sections/map.rs` | NEW - MapPreview, TiledMapSection |
| `crates/mimir-dm-print/src/sections/tokens.rs` | NEW - TokenCutoutSheet |
| `crates/mimir-dm-print/src/sections/monsters.rs` | NEW - MonsterAppendix |
| `crates/mimir-dm-print/src/sections/npcs.rs` | NEW - NpcAppendix |
| `crates/mimir-dm-print/src/campaign.rs` | Deprecate old methods, delegate to builder |
| `crates/mimir-dm/src/commands/print/mod.rs` | Update to use DocumentBuilder |

## Status Updates

*To be added during implementation*
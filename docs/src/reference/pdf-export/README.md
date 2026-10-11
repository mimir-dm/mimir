# PDF Export

Mimir makes PDFs with [Typst](https://typst.app/), a typesetting system. The `mimir-print` crate compiles the Typst source inside the server. The server answers with the PDF (`application/pdf`), and the browser downloads it.

## Export Types

| Export | Contents | Where |
|--------|----------|-------|
| Character | Character sheet, battle card, spell cards, equipment cards | **PDF…** on the character sheet ([how-to](../../how-to/characters/print-character-sheet.md)) |
| Document | One document, from its markdown | **PDF** on an open document ([how-to](../../how-to/campaigns/manage-documents.md)) |
| Campaign | Campaign documents, module content, NPCs, map previews, tiled maps and token cutouts, chosen per part | **More**, **PDF…** on the campaign dashboard |
| Book | The campaign as one book: a title page (authors, levels), contents, the campaign documents, each module with its documents and the maps no document shows, the appendices, and the stat blocks and items the text names | **More**, **Print as a book…** on the campaign dashboard |
| Module | Documents, monsters, traps, points of interest, NPCs, play notes, map previews and tiled maps, chosen per part | **More**, **PDF…** on the module page |
| Map | A preview page and/or tiled pages at true scale, with token cutouts | **PDF…** on the DM map ([how-to](../../how-to/maps/print-map.md)) |
| Monster cards | A half-page card for each monster of a module | **More**, **Monster cards (PDF)** on the module page |

Only the DM can export PDFs.

## Options

The PDF dialog lists the options of each export. Each option is a query parameter of the route.

| Export | Options (default on in bold) |
|--------|------------------------------|
| Character | **Character sheet**, **Battle card**, Spell cards, Equipment cards |
| Campaign | **Campaign documents**, **Module content**, **NPCs**, Module map previews, Module maps to play on (tiled), Token cutouts, Campaign map previews, Campaign maps to play on (tiled) |
| Module | **Documents**, **Monsters**, **Traps**, **Points of interest**, **NPCs**, Play notes, **Map previews**, **Previews: grid**, Previews: walls, Previews: positions, Maps to play on (tiled), **Play: grid**, Play: walls, Play: token cutouts |
| Map | **Preview (one page)**, **Preview: grid**, Preview: walls, Preview: positions, To play on (tiled, actual size), **Play: grid**, Play: walls, Play: token cutouts |

## API

The routes are under `/api/v1` and need the DM token.

| Route | Query options |
|-------|---------------|
| `GET /characters/{id}/pdf` | `include_compact_sheet`, `include_battle_card`, `include_spell_cards`, `include_equipment_cards` |
| `GET /campaigns/{id}/pdf` | `include_campaign_docs`, `include_module_content`, `include_npcs`, `include_module_map_previews`, `include_module_tiled_maps`, `include_token_cutouts`, `include_campaign_map_previews`, `include_campaign_tiled_maps` |
| `GET /campaigns/{id}/book` | `authors` (comma-separated), `level_start`, `level_end` |
| `GET /modules/{id}/pdf` | `include_documents`, `include_monsters`, `include_traps`, `include_pois`, `include_npcs`, `include_play_notes`, `include_preview`, `preview_grid`, `preview_los_walls`, `preview_positions`, `include_play`, `play_grid`, `play_los_walls`, `play_cutouts` |
| `GET /maps/{id}/pdf` | `include_preview`, `preview_grid`, `preview_los_walls`, `preview_positions`, `include_play`, `play_grid`, `play_los_walls`, `play_cutouts` |
| `GET /documents/{id}/pdf` | none |
| `GET /modules/{id}/monster-cards` | `show_cut_lines` |

Each `include_…` option takes `true` or `false`.

In every document PDF, read-aloud boxes and sidebars print as boxes, a stat block line prints the stat block, a tag prints as its words, and an image prints at page width: a 5etools image from the imported images, an uploaded image from its file. An image Mimir cannot find prints as its title in a box.

## Section Types

PDFs are assembled from typed sections (`crates/mimir-print/src/sections/`), each of which produces Typst markup:

| Section | Description |
|---------|-------------|
| Markdown document | Campaign/module documents with optional YAML frontmatter (`title`, `type`) converted from markdown to Typst |
| Character sheet | Basic info, ability scores, classes, and roleplay elements |
| Character battle card | Half-page (3.875" × 5.125") combat reference cards in a 2×2 layout, for PCs and NPCs |
| Monster cards | Half-page (3.875" × 5.125") stat-block cards in a 2×2 layout |
| Spell cards | Multi-up cards; long descriptions split at natural boundaries onto foldable continuation cards |
| Equipment cards | 2.5" × 3.25" cards for weapons, special ammo, and magic items |
| Map preview | Map fit to a single page, with options for grid overlay, line-of-sight walls drawn as red lines, and starting positions as numbered circles |
| Tiled map | Map rendered at true scale across multiple labeled pages for physical play |
| Token cutouts | Fold-in-half paper standees sized by creature category (tiny through gargantuan), with per-token quantity |
| Trap cards | Half-page (3.875" × 5.125") cards showing trigger, effect, countermeasures, and details |

## Rendering Pipeline

Each route gathers data from the database, builds the relevant sections, and assembles them with a `DocumentBuilder`. The combined Typst source is compiled in-memory by the Typst compiler library inside a custom `MimirTypstWorld` — no external Typst CLI and no temporary data files are involved. Images (maps, tokens) are passed to the compiler as in-memory virtual files.

Shared Typst components are embedded in the server binary (`crates/mimir-print/src/embedded_templates.rs`): `_shared/styles.typ`, `_shared/components.typ`, and `_shared/icons.typ`. Imports of these paths are resolved from the embedded copies before any disk lookup. The embedded styles define grayscale, print-friendly colors and the font stacks Inter / Helvetica / Arial (headings and body) and JetBrains Mono / Consolas (monospace).

Fonts are not bundled: the compiler uses the fonts installed on the server, discovered once per process via `fontdb` (`crates/mimir-print/src/world.rs`). If the first font in a stack is not installed, Typst falls back through the stack.

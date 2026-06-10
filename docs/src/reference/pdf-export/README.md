# PDF Export

Mimir generates PDFs from campaign data using [Typst](https://typst.app/), a modern typesetting system compiled in-process by the `mimir-print` crate. Every export returns the finished PDF to the frontend as base64 data, where it can be previewed and saved to disk.

## Export Types

| Export | Contents | Triggered from |
|--------|----------|----------------|
| Character | Full character sheet, plus optional compact sheet, battle card, spell cards, and equipment cards | Character print dialog ([how-to](../../how-to/characters/print-character-sheet.md)) |
| Single document | One campaign document rendered from its markdown content | Document editor ([how-to](../../how-to/campaigns/manage-documents.md)) |
| Campaign | Campaign documents, module content, NPCs, map previews, true-scale tiled maps, and token cutouts, toggled per section | Campaign export dialog |
| Module | Module documents, monster cards, trap cards, points of interest, NPCs, play notes, and map preview/play pages, toggled per section | Module export dialog ([how-to](../../how-to/modules/module-documents.md)) |
| Map | Preview page (fit to one page) and/or true-scale tiled play pages with token cutouts | Map print dialog ([how-to](../../how-to/maps/print-map.md)) |
| Monster cards | Half-page reference cards for one monster, or all monsters in a module | `export_monster_card` / `export_module_monsters` commands |
| Trap cards | Half-page reference cards for one or more traps | `export_trap_card` / `export_trap_cards` commands |

## Tauri Commands

The application registers twelve print commands (`crates/mimir/src/main.rs`), implemented in `crates/mimir/src/commands/print/`.

| Command | Parameters | Description |
|---------|------------|-------------|
| `list_print_templates` | — | List `.typ` files found under the app data `templates/` directory |
| `export_character` | `character_id`, `options?` | Character sheet with optional compact sheet, battle card, spell cards, equipment cards |
| `export_campaign_document` | `document_id` | Render a single campaign document |
| `export_campaign_documents` | `campaign_id`, `options?` | Combined campaign PDF with per-section toggles |
| `export_module_documents` | `module_id`, `options?` | Combined module PDF with per-section toggles |
| `print_map` | `map_id`, `options?` | Map preview and/or tiled play pages |
| `generate_character_sheet` | `character_id`, `template?` | Legacy API; currently returns a "not yet implemented" error |
| `save_pdf` | `pdf_base64`, `path` | Decode base64 PDF data and write it to a file |
| `export_module_monsters` | `module_id`, `options?` | Cards for every monster in a module |
| `export_monster_card` | `monster_name`, `monster_source`, `options?` | Card for a single catalog monster |
| `export_trap_card` | `trap_name`, `trap_source`, `options?` | Card for a single trap |
| `export_trap_cards` | `traps` (list of name/source pairs), `options?` | Cards for multiple traps |

Card commands accept a `show_cut_lines` option. Map and module options include grid overlay, line-of-sight walls, starting positions, and token cutout toggles.

All rendering commands return `ApiResponse<PrintResult>`, where `PrintResult` contains `pdf_base64` (the PDF as base64) and `size_bytes` (the decoded size). `save_pdf` returns `ApiResponse<()>` and `list_print_templates` returns a list of `{ id, name, category }` entries.

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

Each command gathers data from the database, builds the relevant sections, and assembles them with a `DocumentBuilder`. The combined Typst source is compiled in-memory by the Typst compiler library inside a custom `MimirTypstWorld` — no external Typst CLI and no temporary data files are involved. Images (maps, tokens) are passed to the compiler as in-memory virtual files.

Shared Typst components are embedded in the application binary (`crates/mimir-print/src/embedded_templates.rs`): `_shared/styles.typ`, `_shared/components.typ`, and `_shared/icons.typ`. Imports of these paths are resolved from the embedded copies before any disk lookup. The embedded styles define grayscale, print-friendly colors and the font stacks Inter / Helvetica / Arial (headings and body) and JetBrains Mono / Consolas (monospace).

Fonts are not bundled: the compiler uses system fonts, discovered once per process via `fontdb` (`crates/mimir-print/src/world.rs`). If the first font in a stack is not installed, Typst falls back through the stack.

## Template Directory

`list_print_templates` scans the `templates/` directory inside the application data directory, recursively collecting `.typ` files (anything under a `_shared/` directory is skipped). Each entry's `id` is its relative path without the extension, its `category` is the first directory component, and its `name` is the title-cased file name. If the directory does not exist, the command returns an empty list.

This listing is informational only: the export commands above generate their Typst source from the embedded section types and embedded shared components, and do not render user-supplied `.typ` files.

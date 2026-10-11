# Adventure File Format

Mimir reads and writes adventures in the 5etools homebrew format (ADR MIMIR-A-0011). Mimir stores documents as markdown and converts on the way in and out. The [Campaign Archive Format](./campaign-archive-format.md) is a different file: it holds a campaign as it was played.

## The File

A JSON object with these keys:

| Key | Holds |
|-----|-------|
| `_meta` | `sources`: one source with `json` (the source code), `abbreviation`, `full`, `authors`, `version`, an optional `url`, and `convertedBy: ["Mimir"]`; `dateAdded` and `dateLastModified` (seconds); `edition: "classic"` |
| `adventure` | One entry: `name`, `id` and `source` (the source code), `group: "homebrew"`, `published` (the export date), `author`, `storyline`, `level` (`start`, `end`) and `contents` |
| `adventureData` | One entry: `id`, `source` and `data`, the sections of the adventure |
| `monster`, `monsterFluff`, `item`, `baseitem`, `spell`, `condition`, `status`, … | The entries the adventure carries, each under the adventure's source |

`contents` has an entry for each section of `data`, with its `name` and `headers` (the names of its parts). A module has `ordinal: {"type": "chapter", "identifier": n}`, an appendix `ordinal: {"type": "appendix", "identifier": "A"}`.

## Reading

Mimir reads the `adventureData` wrapper (current 5etools data and homebrew) and the older top-level `data`. A homebrew file's source must not be in the catalog already.

## Rules of the Source Code

The vendored 5etools schema (`crates/mimir-core/schema/5etools/util.json`, `sourceJson`) asks for:

- 6 or more characters;
- letters, digits, `-`, `&`, `+`, `!`, and spaces inside;
- not a code that starts with `UA` or `XUA`, and not the code of an official source.

## Converting Markdown

| 5etools entry | Mimir markdown |
|---------------|----------------|
| `section`, named `entries` | A heading; an `id` that an `{@area}` names as `{#id}` after the heading |
| `insetReadaloud` | `> [!read-aloud]` and the quoted text |
| `inset` | `> [!sidebar] Title` and the quoted text |
| `quote` | A block quote, `— by` on its last line |
| `list`, `item` | A list; `- **Name** text` for a named item |
| `table` | A table, with a `Table: caption` line before it |
| `image` | `![title](5etools:path)` for a 5etools image |
| `statblock` | A line that is only the tag: `{@creature Name\|Source}`; `{@creatureFluff Name\|Source}` for a creature's lore |

A conversion loses list styles, column widths, a table cell's roll (it becomes its text), image sizes and grids, and `id`s that no `{@area}` names.

## See Also

- [Export a Campaign as an Adventure](../how-to/campaigns/export-adventure.md)
- [Make a Campaign from an Adventure](../how-to/campaigns/campaign-from-adventure.md)

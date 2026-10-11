# Export a Campaign as an Adventure

Mimir can write a campaign as a 5etools homebrew adventure file. 5etools users load the file to read the adventure. Mimir reads it too: see [Import a Homebrew File](./manage-sources.md#import-a-homebrew-file).

To give another Mimir user the maps with their walls and the placements on them, use a campaign archive instead. See [Export a Campaign](./export-campaign.md).

## Export the File

1. Open the campaign.
2. Click **More**, then **Export as a 5etools adventure…**.
3. Fill in the dialog:
   - **Source code**: the adventure's own source, 6 or more letters, digits or `- & + !`, not the code of an official book. For example `LowFen`.
   - **Adventure name**: the campaign's name by default.
   - **Authors**: separate names with commas.
   - **Version**, **Storyline**, **From level**, **To level**, and optionally a **Web address**.
4. Click **Download**.

The browser downloads `<source code>.json`.

## What the File Holds

| In the campaign | In the file |
|-----------------|-------------|
| Campaign documents | Sections before the chapters. A document whose title starts with "Appendix", or "Credits", comes after them. |
| Modules | Chapters, in module order. Each document of a module is a named part of its chapter. |
| Read-aloud boxes, sidebars, stat block lines, tags | The same 5etools entries the reader shows |
| A link between documents that Mimir made | An area link (`{@area}`) |
| The campaign's homebrew monsters, items and spells | Entries of the file |
| Every monster, item, spell, condition and other catalog entry that the text or a module names | A copy in the file, under the adventure's source code. The text's tags point at the copies, so the file reads with no rule books loaded. |

Skills, senses and decks have no catalog entry in Mimir: their tags keep their official source.

## With Claude

The MCP tool `export_adventure_file` exports the active campaign. See [MCP Server Reference](../../reference/mcp-server.md).

## See Also

- [Adventure File Format](../../reference/adventure-file-format.md)
- [Make a Campaign from an Adventure](./campaign-from-adventure.md)

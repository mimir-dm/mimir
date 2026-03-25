# The Catalog System

How Mimir manages D&D 5e reference data and why the catalog works the way it does.

## Two Kinds of Data

Mimir's database holds two fundamentally different kinds of content:

**Catalog data** is reference material — monsters, spells, items, races, classes, backgrounds, feats, conditions, and more. This data is read-only within Mimir. You search it, browse it, and reference it, but you don't edit it directly.

**Campaign data** is your content — campaigns, modules, characters, documents, maps, homebrew. This is fully editable and belongs to you.

The catalog exists to support the campaign. When you add a monster to a module, you're creating a reference from your campaign data to the catalog. When you add a spell to a character, same thing. The catalog is the shared dictionary; your campaign is the story you write with it.

## Import, Not Distribution

Mimir doesn't ship with D&D content — it can't, for licensing reasons. Instead, it provides an import system that reads structured data archives and populates the catalog tables.

The import format follows the schema established by the [5etools](https://5e.tools/) community project. If you have a compatible data archive, Mimir can import it. The importer handles:

- Monsters with full stat blocks, traits, and actions
- Spells with casting details and class lists
- Items including weapons, armor, and magic items with variant generation
- Races, classes, subclasses, backgrounds, feats
- Conditions, languages, deities, and many other entity types
- Token images and creature artwork
- Source book metadata and publication information

After import, all data lives in your local SQLite database. Mimir never phones home or downloads content.

## Source Filtering

Not every campaign uses every sourcebook. A "Player's Handbook only" game shouldn't show Xanathar's Guide spells in character creation.

Mimir's source filtering works at two levels:

**Campaign sources** control which books are active for a campaign. When you open Campaign Sources and enable only PHB and MM, catalog searches within that campaign only return content from those books.

**Character sources** optionally restrict further. A character can have their own source list — useful when one player owns different books than another.

This filtering is consistent everywhere: spell searches, monster searches, item searches, and character creation all respect the active source configuration.

## Full-Text Search

The catalog uses SQLite's FTS5 (Full-Text Search) engine. When you search for "fire," it matches:

- Monster names: "Fire Elemental", "Fire Giant"
- Spell names: "Fireball", "Fire Bolt"
- Item descriptions: a sword with fire damage
- Monster abilities that mention fire resistance

The FTS index covers both rule text and flavor text, using the Porter stemming algorithm so "casting" matches "cast" and "dragons" matches "dragon."

## Homebrew Integration

Homebrew content (custom items, monsters, spells) lives alongside catalog data in search results. When you search for monsters to add to a module, homebrew monsters from the active campaign appear in the same results list, marked with an **HB** badge.

This blending is intentional — you shouldn't have to remember whether that custom dragon variant is homebrew or catalog. It just shows up where you need it.

See [The Homebrew System](./homebrew-system.md) for details on creating custom content.

## Data Structure

Catalog entities are stored as structured JSON in the database, following the schema conventions of the source data format. This means:

- Monster stat blocks include everything: abilities, actions, legendary actions, lair actions, regional effects
- Spells include casting components, duration, scaling behavior
- Items include variant generation rules for magic items (a +1/+2/+3 weapon generates three entries)

The JSON is parsed on-demand by the frontend formatters, which render stat blocks, spell descriptions, and item cards in familiar D&D presentation style.

## Why Not a Bundled Database?

You might wonder why Mimir doesn't ship with a pre-populated catalog. Three reasons:

1. **Licensing.** D&D content is owned by Wizards of the Coast. Distributing it would require a license Mimir doesn't have.
2. **Freshness.** The community data format is actively maintained and updated. Importing lets you use the latest corrections and additions.
3. **Choice.** You control exactly which sources are in your database. No bloat from content you don't use.

## See Also

- [Manage Campaign Sources](../how-to/campaigns/create-campaign.md) — Configuring which books are active
- [The Homebrew System](./homebrew-system.md) — Creating custom content
- [File Formats](../reference/file-formats.md) — Technical details on data formats

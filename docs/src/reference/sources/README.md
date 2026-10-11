# Books and Sources

Mimir supports importing D&D 5e source books for reference and catalog population. The source system allows you to browse book content, search the extracted catalog data, and navigate cross-references between game elements.

Mimir ships with no game content — not even the System Reference Document (SRD). All catalog data comes from archives you import. See [The Catalog System](../../explanation/catalog-system.md) for why, and [Manage Campaign Sources](../../how-to/campaigns/manage-sources.md) for the import procedure.

## Archive Format

Source data is imported from gzip-compressed tar archives (`.tar.gz`) containing JSON data in the format established by the [5etools](https://5e.tools/) community project. The archive must include a `books.json` file describing the available sources; the importer streams the archive directly without extracting it to disk.

Pre-packaged archives are available from the [Mimir Resources](https://github.com/mimir-dm/resources/releases) releases page. Two archive types are published:

| Archive | Contents |
|---------|----------|
| Data archive | 5etools-format JSON (books, spells, monsters, items, and other entities) |
| Image archive | Token art, cover art, and book illustrations (`5etools-img-*` prefixed paths) |

The importer only processes source books whose group is `core` (PHB, DMG, MM) or `supplement` (XGE, TCE, etc.). Books in other groups — setting books (Eberron, Ravnica, Theros), adventures, DM screens, homebrew — are skipped.

## Catalog Population

When you import a data archive, Mimir extracts game content into catalog tables in the local SQLite database. Spells go into the `spells` table, monsters into `monsters`, items into `items`, and so on for classes, races, feats, backgrounds, conditions, actions, and other entity types. Magic item variants are expanded during import (a +1/+2/+3 weapon template generates an entry per base item).

The catalog tables store both summary information for search results and complete details for individual entries, allowing fast searching across thousands of entries while still providing full stat blocks and descriptions for a specific entry.

## Cross-Reference System

Book content contains tagged references using the `{@type name|source}` syntax. A reference like `{@spell fireball|PHB}` identifies a spell named "fireball" from the Player's Handbook. The content renderer converts these tags into clickable links that display tooltips on hover and full details on click.

The cross-reference system supports spells, creatures, items, conditions, actions, classes, races, feats, and backgrounds. References to content from sources you haven't imported appear as plain text rather than links.

## Storage Location

Catalog data lives in the SQLite database of the server: `<data dir>/data/mimir.db`. Imported images go to `<data dir>/assets/catalog/`. The server serves them on demand. See [Environment Variables](../environment-variables.md#mimir_data_dir).

## Where to Use It

- **Settings**, **Books**: import data and images, remove a source. See [Settings](../ui/settings.md).
- **Reference**: search the catalog and read the books. See [Reference Page](../ui/reference-page.md).
- **Source books…** on a campaign or a character: choose the books that its searches use.

## API

The web app uses these routes under `/api/v1`. They need the DM token.

| Method and path | Function |
|-----------------|----------|
| `POST /catalog/import` | Upload a 5etools data archive. Answers with a job. |
| `POST /catalog/import-images` | Upload a 5etools image archive. Answers with a job. |
| `GET /jobs/{id}` | The state of an import job |
| `GET /catalog/sources` | The imported sources |
| `DELETE /catalog/sources/{code}` | Remove a source and its catalog entries |
| `GET /catalog/search/{category}` | Search one kind of entry |
| `GET /catalog/entry/{category}/{source}/{name}` | One entry, rendered |
| `GET /books/{source}` | The text of a book |
| `GET /book-images/{path}` | An image of a book |
| `GET`, `PUT /campaigns/{id}/sources` | The source books of a campaign |
| `GET`, `PUT /characters/{id}/sources` | The source books of a character |

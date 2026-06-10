# Campaign Archive Format

Technical reference for Mimir's campaign export/import archive format.

## Overview

Campaign archives are gzip-compressed tar files with the extension `.mimir-campaign.tar.gz`. An archive contains a complete snapshot of one campaign — metadata, modules, documents, characters, maps, tokens, homebrew content, and binary assets. Archives are self-contained and portable between machines and Mimir installations.

| Property | Value |
|----------|-------|
| Format identifier | `mimir-campaign` |
| Format version | `2.0` |
| File extension | `.mimir-campaign.tar.gz` |
| Export filename | `<campaign-name-slug>.mimir-campaign.tar.gz` (lowercased, non-alphanumeric characters replaced with hyphens; consecutive separators collapsed to a single hyphen; leading and trailing hyphens stripped) |

## Archive Structure

```
<slug>.mimir-campaign.tar.gz
├── manifest.json                    # Archive metadata
├── data.json                        # All campaign data
└── assets/
    └── {asset-id}/{filename}        # One file per campaign asset
```

The archive contains exactly two JSON files plus one entry per asset. Asset files are stored under their original asset ID and filename; binary content is not hashed or deduplicated.

## Manifest

`manifest.json` contains archive metadata:

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Archive format version (`"2.0"`) |
| `format` | string | Format identifier (`"mimir-campaign"`) |
| `created_at` | string | ISO 8601 export timestamp (UTC) |
| `campaign_name` | string | Campaign name at export time |
| `mimir_version` | string | Mimir version that created the archive |
| `counts` | object | Entity counts (see below) |
| `catalog_references` | array | Catalog items referenced by campaign documents |

### Counts

| Field | Type |
|-------|------|
| `modules` | number |
| `documents` | number |
| `characters` | number |
| `maps` | number |
| `tokens` | number |
| `module_monsters` | number |
| `module_npcs` | number |
| `assets` | number |
| `homebrew_items` | number |
| `homebrew_monsters` | number |
| `homebrew_spells` | number |

### Catalog References

Each entry identifies a catalog item that campaign documents reference but the archive does not contain:

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Reference type: `monster`, `spell`, `item`, `creature`, `condition`, `feat`, `race`, `class`, or `background` |
| `name` | string | Item name |
| `source` | string | Source book code |

References are extracted at export time by scanning document content for `{@type name|source}` tags.

## Campaign Data

`data.json` contains all campaign entities in a single JSON object:

| Field | Type | Description |
|-------|------|-------------|
| `campaign` | object | Campaign record |
| `sources` | string[] | Enabled source book codes |
| `modules` | array | Module records |
| `documents` | array | Document records (full text content) |
| `characters` | array | Characters with related data (see below) |
| `maps` | array | Maps with related data (see below) |
| `tokens` | array | Token placements across all maps |
| `module_monsters` | array | Monster references attached to modules |
| `module_npcs` | array | NPCs attached to modules |
| `assets` | array | Asset records (filename, MIME type, blob path) |
| `homebrew_items` | array | Campaign homebrew items |
| `homebrew_monsters` | array | Campaign homebrew monsters |
| `homebrew_spells` | array | Campaign homebrew spells |

### Character Entries

Each character entry contains the base character fields (flattened) plus:

| Field | Contents |
|-------|----------|
| `classes` | Class entries (levels, subclasses, starting-class flag) |
| `inventory` | Inventory items |
| `spells` | Known spells |
| `proficiencies` | Proficiency entries |
| `features` | Class features |
| `feats` | Feats |

### Map Entries

Each map entry contains the base map fields (flattened) plus:

| Field | Contents |
|-------|----------|
| `pois` | Points of interest |
| `traps` | Trap placements |
| `light_sources` | Light source definitions |
| `fog_areas` | Fog of war revealed areas |

## Asset Files

Binary files (map images, token art) are stored in the archive at `assets/{asset-id}/{filename}`, where `{asset-id}` is the asset's UUID at export time and `{filename}` is its original filename.

## Import Behavior

When an archive is imported:

1. The archive is extracted and `data.json` is parsed; import fails if `data.json` is missing or unparseable. The manifest's `format` field is not checked during import — that validation happens only in archive preview (see [Preview](#preview))
2. New UUIDs are generated for all entities, so imports never conflict with existing data
3. Internal references (module, document, character, map, asset, and monster/NPC IDs) are remapped to the new UUIDs
4. Asset files are copied into the importing installation's assets directory under `campaigns/{new-campaign-id}/{filename}`
5. Campaign sources are restored; they take effect only if the corresponding catalog data exists in the importing installation
6. If the campaign name already exists, a numeric suffix is appended (e.g., `Name (2)`), unless a name override is supplied

Importing the same archive twice creates two independent copies of the campaign.

## Preview

An archive preview reads `manifest.json` without extracting or importing the rest of the archive. Preview is where the manifest's `format` field is validated: archives whose `format` is not `"mimir-campaign"` are rejected. The preview contains:

| Field | Description |
|-------|-------------|
| `campaign_name` | From manifest |
| `counts` | Entity counts from manifest |
| `catalog_references` | Catalog items the campaign references |
| `mimir_version` | Mimir version that created the archive |
| `created_at` | Export timestamp |
| `archive_version` | Archive format version |

## See Also

- [Export Campaign](../how-to/campaigns/export-campaign.md) — How to create archives
- [Data Model](./data-model.md) — Entity relationships

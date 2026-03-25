# Campaign Archive Format

Technical reference for Mimir's campaign export/import archive format.

## Overview

Campaign archives are `.tar.gz` files containing a complete snapshot of a campaign — metadata, modules, characters, documents, maps, homebrew, and assets. Archives are self-contained and portable between machines and Mimir installations.

## Archive Structure

```
campaign-export/
├── manifest.json           # Archive metadata
├── campaign.json           # Campaign data
├── characters/
│   ├── {character-id}.json # One file per character
│   └── ...
├── modules/
│   ├── {module-id}.json    # One file per module
│   └── ...
├── documents/
│   ├── {document-id}.json  # One file per document
│   └── ...
├── maps/
│   ├── {map-id}.json       # Map metadata and overlays
│   └── ...
├── homebrew/
│   ├── items/
│   │   └── {item-id}.json
│   ├── monsters/
│   │   └── {monster-id}.json
│   └── spells/
│       └── {spell-id}.json
└── assets/
    ├── {file-hash}         # Binary assets (maps, images)
    └── ...
```

## Manifest

The `manifest.json` file contains archive metadata:

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Archive format version |
| `created_at` | string | ISO 8601 export timestamp |
| `campaign_id` | string | Original campaign UUID |
| `campaign_name` | string | Campaign name at export time |
| `mimir_version` | string | Mimir version that created the archive |

## Campaign Data

The `campaign.json` file contains:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Campaign UUID |
| `name` | string | Campaign name |
| `description` | string | Campaign description |
| `sources` | string[] | Enabled source book codes |
| `created_at` | string | Original creation timestamp |
| `updated_at` | string | Last modification timestamp |

## Character Files

Each character is a separate JSON file containing:

- Base character data (name, race, class, ability scores, HP, AC)
- Class entries (multiclass levels and subclasses)
- Inventory items with quantity, equipped, and attuned state
- Known spells with prepared state and granting class
- Proficiencies (skills, tools, languages, saving throws)
- Feats and class features

## Module Files

Each module JSON file contains:

- Module metadata (name, description, type, sort order)
- Monster references (catalog monster names, quantities, notes)
- NPC data (name, role, description, stat block)
- Token placements (positions, visibility, faction colors)

## Document Files

Each document JSON file contains:

- Document metadata (title, type, sort order)
- Parent reference (campaign_id or module_id)
- Full text content

## Map Files

Each map JSON file contains:

- Map metadata (name, lighting mode, fog state)
- Light source definitions (positions, colors, radii)
- Fog of war state (revealed areas)
- Points of interest (positions, descriptions)
- Trap placements (positions, detection info)
- Reference to the map asset file (by hash)

## Asset Files

Binary files (map images, token art) stored by their SHA-256 content hash. This deduplicates assets — if two maps use the same image, it's stored once.

## Import Behavior

When importing an archive:

1. **New UUIDs** are generated for all entities to avoid conflicts with existing data
2. **Internal references** (e.g., module_id on documents) are remapped to the new UUIDs
3. **Assets** are copied to the importing machine's assets directory
4. **Campaign sources** are restored but only take effect if the corresponding catalog data has been imported
5. The imported campaign appears in the campaign list as a new entry

Importing the same archive twice creates two independent copies of the campaign.

## Preview

The `preview_archive` tool/command reads the manifest and entity counts without performing a full import. Use this to inspect an archive before importing:

| Field | Description |
|-------|-------------|
| Campaign name | From manifest |
| Module count | Number of module files |
| Character count | Number of character files |
| Document count | Number of document files |
| Map count | Number of map files |
| Homebrew counts | Items, monsters, spells |
| Archive size | Total file size |

## See Also

- [Export Campaign](../how-to/campaigns/export-campaign.md) — How to create archives
- [Data Model](./data-model.md) — Entity relationships

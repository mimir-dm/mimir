# Mimir Source Data Format

This directory documents the source data archive format that Mimir expects for rendering D&D 5e content. Each archive contains structured JSON files following specific schemas for reliable parsing and display.

## Overview

Mimir consumes source data as compressed archives (`.tar.gz` files), with each archive representing a single D&D sourcebook. The archives follow a standardized directory structure and JSON schema format to ensure consistent rendering across all content types.

## Archive Format

- **File Format**: TAR archive compressed with GZIP (`.tar.gz`)
- **Naming Convention**: `{book-abbreviation}.tar.gz` (lowercase)
- **Examples**: `phb.tar.gz`, `mm.tar.gz`, `dmg.tar.gz`

## Documentation Structure

```
source-data/
├── README.md                     # This file - overview of source data format
├── archive-structure.md          # Complete archive directory layout
└── schemas/                      # JSON schema specifications
    ├── content/                  # Content type schemas
    │   ├── spell-schema.md       # Spell data format
    │   ├── equipment-schema.md   # Items and equipment format
    │   ├── magic-items-schema.md # Magic item properties
    │   ├── bestiary-schema.md    # Monster and creature format
    │   ├── class-schema.md       # Classes and subclasses
    │   ├── race-schema.md        # Races and subraces
    │   ├── background-schema.md  # Character backgrounds
    │   └── feat-schema.md        # Character feats
    └── metadata/                 # Metadata and configuration
        ├── metadata-schema.md    # Book metadata format
        └── book-schema.md        # Book content structure
```

## Quick Reference

### Essential Files in Every Archive

1. **`metadata.json`** - Book identification and configuration
2. **`book/book-{source}.json`** - Table of contents and structure
3. **Content directories** - Organized by type (spells/, items/, bestiary/, etc.)

### Content Organization

Each content type is stored in its own directory with consistent naming:

- **Primary Data**: `{type}-{source}.json` - Game mechanics and rules
- **Fluff Data**: `fluff-{type}-{source}.json` - Lore and descriptions
- **Images**: `img/` directory with subdirectories for different image types

### Common Patterns

All JSON files follow these patterns:

1. **Root Object**: Contains a single key matching the content type
   ```json
   {
     "spell": [...],     // For spells
     "item": [...],      // For items
     "monster": [...]    // For monsters
   }
   ```

2. **Source References**: Cross-references use `name|source` format
   ```json
   "baseItem": "longsword|phb"
   ```

3. **Required Fields**: Every entry has at minimum:
   - `name` - Display name
   - `source` - Source book code
   - `page` - Page number reference

## Getting Started

1. **[Archive Structure](./archive-structure.md)** - Start here to understand the complete directory layout
2. **[Content Schemas](./schemas/content/)** - Detailed specifications for each content type
3. **[Metadata Schemas](./schemas/metadata/)** - Book configuration and structure

## Validation

Archives should be validated to ensure:

- Correct directory structure
- Valid JSON syntax
- Schema compliance for all content types
- Cross-reference integrity
- Required files present

## Examples

For reference implementations, examine the processed archives for core books:
- `phb.tar.gz` - Player's Handbook (classes, races, spells, equipment)
- `mm.tar.gz` - Monster Manual (bestiary content)
- `dmg.tar.gz` - Dungeon Master's Guide (magic items, variant rules)
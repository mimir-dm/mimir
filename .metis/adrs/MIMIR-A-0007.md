---
id: 001-uvtt-as-internal-map-storage-format
level: adr
title: "UVTT as Internal Map Storage Format"
number: 1
short_code: "MIMIR-A-0007"
created_at: 2025-12-29T15:08:05.115194+00:00
updated_at: 2025-12-29T15:08:55.851212+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: UVTT as Internal Map Storage Format

*This template includes sections for various types of architectural decisions. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

Mimir needs to store battle maps with associated metadata for VTT-style display during in-person D&D sessions. Maps require:

- High-resolution images (often 4K+)
- Grid configuration (size, offset, type)
- Line-of-sight (LOS) walls for fog of war
- Portals (doors) that can be opened/closed
- Light sources with range and color
- Ambient lighting settings

Previously, maps were stored as processed JPEG images with grid metadata in the database. This approach lacked support for LOS walls, portals, and lights—features essential for dynamic fog of war and lighting systems.

## Decision **[REQUIRED]**

Use Universal VTT (.dd2vtt) format as the internal storage format for all maps. When users upload:

1. **UVTT files**: Store directly, extract metadata for database
2. **Plain images**: Wrap in UVTT structure with default grid (70px), empty LOS/portals/lights

All maps are stored as `.dd2vtt` JSON files containing:
- Base64-encoded image
- Grid resolution and origin
- Line-of-sight wall segments
- Portal definitions (doors with open/closed state)
- Light source positions, ranges, and colors
- Environment settings (ambient light level)

Storage paths:
- Campaign maps: `{data_dir}/campaigns/{campaign_id}/maps/{uuid}.dd2vtt`
- Module maps: `{data_dir}/modules/{module_id}/maps/{uuid}.dd2vtt`

## Rationale **[REQUIRED]**

1. **Industry standard**: UVTT is widely supported by map-making tools (Dungeondraft, Dungeon Alchemist, etc.)
2. **Single source of truth**: All map data (image + metadata) in one file
3. **Forward compatible**: Plain image uploads wrapped in UVTT can later have LOS/lights added
4. **No conversion loss**: UVTT stores original image as base64 PNG, no quality degradation
5. **Portable**: Maps can be exported and used in other VTT systems

## Consequences **[REQUIRED]**

### Positive
- Full LOS/portal/lighting support without separate storage
- Compatible with popular map-making tools
- Users can import pre-made UVTT maps with all metadata intact
- Enables dynamic fog of war and lighting in the DM view
- Single file backup/restore for maps

### Negative
- Larger file sizes (base64 encoding adds ~33% overhead)
- JSON parsing required to extract image for display
- Cannot stream image data (must load full file)

### Neutral
- Database stores only map ID, name, dimensions, grid config (metadata extracted from UVTT)
- Legacy JPEG maps in `{data_dir}/maps/` still supported via path detection
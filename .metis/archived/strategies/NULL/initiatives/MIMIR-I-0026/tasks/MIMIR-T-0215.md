---
id: create-light-sources-database
level: task
title: "Create light sources database schema and model"
short_code: "MIMIR-T-0215"
created_at: 2025-12-22T14:40:31.645927+00:00
updated_at: 2025-12-22T14:54:56.411286+00:00
parent: MIMIR-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0026
---

# Create light sources database schema and model

## Parent Initiative
[[MIMIR-I-0026]] - Vision and Lighting System

## Objective
Create the database migration and Rust models for light sources, token vision fields, and map ambient lighting.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration 039_create_light_sources with up.sql and down.sql
- [ ] `light_sources` table with: map_id, token_id (optional), name, light_type, x, y, bright_radius_ft, dim_radius_ft, color, is_active
- [ ] `tokens` table gains vision_type and vision_range_ft columns
- [ ] `maps` table gains ambient_light column (bright/dim/darkness)
- [ ] Diesel schema.rs updated with all new tables/columns
- [ ] Rust model file: light_sources.rs with LightSource, NewLightSource, UpdateLightSource
- [ ] LightType enum: torch, lantern, candle, spell, custom
- [ ] VisionType enum: normal, darkvision, blindsight, tremorsense, truesight, devils_sight
- [ ] AmbientLight enum: bright, dim, darkness
- [ ] Update Map and Token models with new fields
- [ ] cargo check passes

## Implementation Notes

### Database Schema

**light_sources table:**
- `id` - Primary key
- `map_id` - FK to maps (CASCADE delete)
- `token_id` - Optional FK to tokens (for attached lights that move with token)
- `name` - Display name (e.g., "Torch", "Lantern")
- `light_type` - Enum string: torch, lantern, candle, spell, custom
- `x, y` - Position in pixels (ignored if attached to token)
- `bright_radius_ft` - Bright light radius in feet (default 20 for torch)
- `dim_radius_ft` - Dim light radius in feet (default 40 for torch)
- `color` - Optional hex color tint
- `is_active` - Whether light is currently on

**Token vision fields:**
- `vision_type` - normal, darkvision, blindsight, etc.
- `vision_range_ft` - Range in feet (60 for standard darkvision)

**Map ambient light:**
- `ambient_light` - Base light level: bright, dim, darkness

### Technical Notes
- Light radii stored in feet, converted to pixels using grid size at render time
- Standard 5e grid is 5ft per square, so a 70px grid = 14px per foot
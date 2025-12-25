---
id: vision-and-lighting-system
level: initiative
title: "Vision and Lighting System"
short_code: "MIMIR-I-0026"
created_at: 2025-12-22T14:39:38.422092+00:00
updated_at: 2025-12-22T14:40:53.937416+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: vision-and-lighting-system
---

# Vision and Lighting System Initiative

Extends the Visual Display System with D&D 5e-compliant vision and lighting mechanics.

## Context

The current fog of war system is binary (revealed/hidden). D&D 5e has nuanced lighting and vision rules that affect gameplay:
- Three light levels: Bright, Dim, Darkness
- Vision types: Normal, Darkvision, Blindsight, Tremorsense, Truesight
- Light sources: Torches, lanterns, spells (Light, Darkness, etc.)

DMs need to track what each creature can see based on their vision capabilities and available light.

## Goals & Non-Goals

**Goals:**
- Implement D&D 5e light levels (bright, dim, darkness)
- Support common vision types (darkvision, blindsight)
- Allow placing light sources on map and attached to tokens
- Show appropriate vision for player display based on party vision
- DM can see all areas with lighting indicators

**Non-Goals:**
- Line-of-sight/wall blocking (future initiative)
- Dynamic shadow casting
- Per-player individual displays (all players see same display)

## Detailed Design

### Light Levels
- **Bright Light**: Full visibility, normal Perception
- **Dim Light**: Lightly obscured, shown as slight darkening
- **Darkness**: Heavily obscured, shown as fog (unless darkvision)

### Light Sources
Database table for placed lights with:
- Position (x, y)
- Bright radius, Dim radius
- Color (optional tint)
- Attached to token (optional) - moves with token

### Token Vision
Add to tokens table:
- vision_type: normal, darkvision, blindsight, tremorsense, truesight
- vision_range: distance in feet (for darkvision etc.)

### Rendering
- DM View: Shows all areas, with lighting level indicators
- Player View: Composite of all PC token vision capabilities

## Implementation Plan

**Phase 1: Light Sources**
- Database schema for light sources
- LightSourceService CRUD
- Light source placement UI
- Render light radii on map

**Phase 2: Token Vision**
- Add vision fields to tokens
- Vision configuration in token setup
- Calculate visible areas per token

**Phase 3: Composite Rendering**
- Merge PC vision for player display
- Show dim/dark areas appropriately
- Darkness spell support (magical darkness)
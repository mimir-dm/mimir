---
id: update-mapgen-skill-and-yaml
level: task
title: "Update mapgen skill and YAML reference for room declarations"
short_code: "MIMIR-T-0594"
created_at: 2026-03-11T23:56:33.114332+00:00
updated_at: 2026-03-12T01:04:07.958874+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Update mapgen skill and YAML reference for room declarations

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Update the mapgen skill (`plugin/skills/mapgen/SKILL.md`) and YAML reference (`references/yaml-config-ref.md`) to document room and corridor declarations. Teach the creative director agent how to translate scene descriptions like "guard room connected to a throne room via a locked door" into room config YAML.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `references/yaml-config-ref.md` updated with full `rooms` and `corridors` schema documentation
- [ ] `SKILL.md` updated with room declaration workflow — how the creative director translates "guard room with a door to the throne room" into YAML
- [ ] "Translating Descriptions to Parameters" table updated with room-related entries (e.g., "guard room" → room config, "locked door" → portal type)
- [ ] Example YAML config showing a mixed outdoor+dungeon map (forest with a ruin containing rooms)
- [ ] Scope section updated to reflect that rooms ARE now supported (outdoor-only caveat removed)
- [ ] `/generate-map` command docs updated if needed

## Implementation Notes

- Files: `plugin/skills/mapgen/SKILL.md`, `plugin/skills/mapgen/references/yaml-config-ref.md`, `plugin/commands/generate-map.md`
- This is the final task — depends on all implementation tasks being complete so the docs match reality

## Status Updates

### Completed
- Updated `references/yaml-config-ref.md`:
  - Added full `rooms` schema (id, x, y, width, height, terrain_slot, walls, portals)
  - Added full `corridors` schema (from, from_wall, to, to_wall, width, terrain_slot, portals)
  - Portal types table (door, window, archway, secret_door) with textures and default widths
  - Added wall and portal texture paths to Common Textures section
- Updated `SKILL.md`:
  - Changed title from "Outdoor Terrain" to "Terrain and Room Layout"
  - Replaced "Scope: Outdoor Terrain Only" with "Scope: Terrain and Room Layout" including rooms
  - Added "Room Layout Generation" workflow (6-step process)
  - Added room-related entries to "Translating Descriptions to Parameters" table (guard room, throne room, locked door, secret passage, window, hallway, ruined dungeon)
  - Added room-related config authoring tips
  - Updated description triggers with room/dungeon keywords
  - Replaced "No dungeon/interior support" with "Rooms are declarative" note
- Updated `generate-map.md` command: added rooms/corridors to config discussion, walls/portals to stats output
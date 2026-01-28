---
id: investigate-map-token-system
level: task
title: "Investigate Map/Token System Migration Requirements"
short_code: "MIMIR-T-0411"
created_at: 2026-01-25T01:08:04.690842+00:00
updated_at: 2026-01-25T02:43:50.363381+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# Investigate Map/Token System Migration Requirements

## Parent Initiative

[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective

Analyze the backup (v1) map/token system and the current (v2) implementation to document migration gaps and architectural decisions needed.

## Investigation Findings

### Backup System (v1) - Full Implementation

**Frontend Components:**
- `DmMapViewer.vue` - Main map display (~2162 lines) with full token/light/fog support
- `TokenRenderer.vue` - Token rendering with images, colors, visibility, drag/drop
- `LightSourceRenderer.vue` - Light visualization with bright/dim radii
- `LightOverlay.vue` - UVTT baked lighting overlay

**Composables:**
- `useTokens.ts` - Full token CRUD, image caching, visibility filtering
- `useLightSources.ts` - Light CRUD with presets (torch/lantern/candle)
- `useUvttMap.ts` - UVTT parsing, walls, portals, ambient light
- `useVisibilityPolygon.ts` - Ray-casting LOS algorithm
- `usePlayerDisplay.ts` - IPC synchronization to player window

**Backend Commands (Rust):**
- Token: `create_token`, `list_tokens`, `update_token`, `delete_token`, `toggle_token_visibility`, `serve_token_image`
- Light: `create_light_source`, `list_light_sources`, `update_light_source`, `toggle_light_source`, `delete_light_source`
- Map: `get_map`, `serve_map_image`, `get_uvtt_map`
- Fog: `get_fog_state`, fog persistence

### Current System (v2) - Partial Implementation

**Backend Commands (Implemented):**
- Map: `list_campaign_maps`, `list_module_maps`, `get_map`, `create_map`, `update_map`, `delete_map`, `read_map_uvtt`, `get_uvtt_map`, `serve_map_image`
- Light: `list_light_sources`, `create_light_source`, `toggle_light_source`, `delete_light_source`
- Token: `list_tokens` only (returns TokenWithData with resolved monster/NPC names)

**Backend Commands (MISSING):**
- Token: `create_token`, `update_token`, `delete_token`, `toggle_token_visibility`, `serve_token_image`
- Fog: `get_fog_state`, fog persistence

**Frontend (Partially Migrated):**
- Components exist but reference missing backend commands
- Composables have been started but incomplete

### Architectural Differences

| Aspect | v1 (Backup) | v2 (Current) | Migration Needed |
|--------|-------------|--------------|------------------|
| Token Storage | Diesel ORM | sqlx raw SQL | Schema + repo layer |
| Token Coords | Pixel-based | Pixel-based | Same approach |
| Fog State | Persisted | Not persisted | Add fog_state table |
| Token Images | File-based serving | Not implemented | Asset system |
| UVTT Handling | Full parsing | Full parsing | Already done |
| Player Display | IPC events | Not migrated | Port IPC system |

### Key Decisions Needed

1. **Token Storage Model**: v2 simplified model already defines TOKEN and LIGHT_SOURCE in the initiative. Need to implement repo/service layers.

2. **Fog Persistence**: v1 stored fog_enabled per map. v2 should add to Map table or separate fog_state table.

3. **Token Images**: v1 served from filesystem. v2 should use asset system (same as map images).

4. **Player Display**: IPC synchronization pattern from v1 should be preserved - works well for multi-window VTT.

5. **Vision System**: The useVisibilityPolygon algorithm is complex but battle-tested. Should migrate as-is.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Documented v1 map/token system architecture
- [x] Documented v2 current implementation state
- [x] Identified migration gaps
- [x] Created initiative for map/token migration

## Status Updates

### 2026-01-25
- Completed exploration of backup and current codebases
- Documented findings in this task
- Ready to create migration initiative
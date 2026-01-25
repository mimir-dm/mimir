---
id: uselightsources-composable
level: task
title: "useLightSources composable verification"
short_code: "MIMIR-T-0419"
created_at: 2026-01-25T02:44:21.546231+00:00
updated_at: 2026-01-25T16:04:43.827044+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# useLightSources composable verification

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Verify the useLightSources composable works with backend commands, add runtime state for play session light toggles.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `loadLightSources(mapId)` fetches from backend
- [ ] `createLightSource()` with presets (torch, lantern, candle) works
- [ ] `toggleLightSource()` calls backend and updates local state
- [ ] `deleteLightSource()` works
- [ ] Runtime `is_active` state separate from DB (for play session)
- [ ] Distance conversion helpers (feet ↔ pixels)
- [ ] Computed: `activeLightSources`, `lightsByToken`

## Implementation Notes

### State Architecture

Similar to tokens, separate DB state from runtime state:

```typescript
// DB state (persisted starting configuration)
const dbLightSources = ref<Map<number, LightSource>>(new Map())

// Runtime state (play session toggles - ephemeral)
const runtimeActiveState = ref<Map<number, boolean>>(new Map())

// Merged state
const lightSources = computed(() => {
  return Array.from(dbLightSources.value.values()).map(light => ({
    ...light,
    is_active: runtimeActiveState.value.get(light.id) ?? light.is_active,
  }))
})
```

### Light Presets

```typescript
const LIGHT_PRESETS = {
  torch: { bright_radius_ft: 20, dim_radius_ft: 40, color: '#ff9933' },
  lantern: { bright_radius_ft: 30, dim_radius_ft: 60, color: '#ffcc66' },
  candle: { bright_radius_ft: 5, dim_radius_ft: 10, color: '#ffaa44' },
}
```

### Distance Conversion

```typescript
function feetToPixels(feet: number, gridSize: number): number {
  return (feet / 5) * gridSize  // 5ft per grid square
}
```

### Files to Modify

- `crates/mimir/frontend/src/composables/useLightSources.ts`

### Dependencies

- MIMIR-T-0416 (Light source command updates)

## Status Updates

*To be added during implementation*
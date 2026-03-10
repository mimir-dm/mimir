---
id: token-management-tests-token-list
level: task
title: "Token management tests — token list, create, update, position, vision, visibility"
short_code: "MIMIR-T-0547"
created_at: 2026-03-10T01:31:32.340619+00:00
updated_at: 2026-03-10T01:31:32.340619+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Token management tests — token list, create, update, position, vision, visibility

**Phase 3** — Campaign & Module Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the map token management system — the token palette, token setup modal, token positioning on maps, vision/light calculations, and visibility toggling. These are complex interactive components that are prone to regressions.

## Acceptance Criteria

- [ ] Token palette renders available tokens (module monsters + custom tokens)
- [ ] QuickAddTokenModal creates tokens with correct name, size, and image
- [ ] MapTokenSetupModal configures token properties (vision, light, color)
- [ ] Token placement on map calls correct position update invoke
- [ ] Token visibility toggle updates token state
- [ ] Vision calculation composable produces correct visible area for token position and sight range
- [ ] Light source composable handles bright/dim radius correctly
- [ ] Token removal calls `remove_token` with correct ID
- [ ] All tests pass in CI

## Key Components

- `TokenPalette.vue`
- `QuickAddTokenModal.vue`
- `MapTokenSetupModal.vue`
- `useVisionCalculation.ts` composable
- `useLightSources.ts` composable
- Token position/drag handlers in `DmMapViewer.vue`

## Implementation Notes

The vision and light source composables are pure logic and can be unit tested directly without mounting components. The modal components need invoke mocks for `add_token_to_map`, `remove_token`, `list_tokens_on_map`, `update_map`. Focus on the data flow and state management rather than pixel-perfect canvas rendering.

## Status Updates

*To be added during implementation*
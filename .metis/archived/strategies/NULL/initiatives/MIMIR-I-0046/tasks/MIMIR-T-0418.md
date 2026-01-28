---
id: usetokens-composable-backend
level: task
title: "useTokens composable backend integration"
short_code: "MIMIR-T-0418"
created_at: 2026-01-25T02:44:20.920945+00:00
updated_at: 2026-01-25T16:04:43.508063+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# useTokens composable backend integration

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Connect the useTokens Vue composable to the new Tauri backend commands, managing both DB state (starting positions) and runtime state (play session positions).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `loadTokens(mapId)` fetches from backend via `list_tokens`
- [ ] `createToken()` calls backend `create_token`, updates local state
- [ ] `updateToken()` calls backend `update_token` for setup mode
- [ ] `deleteToken()` calls backend `delete_token`
- [ ] `toggleVisibility()` calls backend `toggle_token_visibility`
- [ ] Runtime position state separate from DB state
- [ ] `loadTokenImage()` calls `serve_token_image`, caches result
- [ ] Computed properties: `visibleTokens`, `tokensByType`

## Implementation Notes

### State Architecture

```typescript
// DB state (starting positions - persisted)
const dbTokens = ref<Map<number, TokenWithData>>(new Map())

// Runtime state (play session - ephemeral)
const runtimePositions = ref<Map<number, { x: number, y: number }>>(new Map())
const deadTokens = ref<Set<number>>(new Set())

// Merged state for rendering
const tokens = computed(() => {
  return Array.from(dbTokens.value.values()).map(token => ({
    ...token,
    x: runtimePositions.value.get(token.id)?.x ?? token.x,
    y: runtimePositions.value.get(token.id)?.y ?? token.y,
    isDead: deadTokens.value.has(token.id),
  }))
})
```

### Key Methods

```typescript
// Setup mode - persists to DB
async function createToken(params: CreateTokenParams): Promise<Token>
async function updateToken(id: number, params: UpdateTokenParams): Promise<Token>
async function deleteToken(id: number): Promise<void>
async function toggleVisibility(id: number): Promise<Token>

// Play mode - runtime only
function moveToken(id: number, x: number, y: number): void  // Updates runtimePositions
function toggleDead(id: number): void  // Updates deadTokens
function resetToStartingPositions(): void  // Clears runtime state

// Image loading
async function loadTokenImage(tokenId: number): Promise<string | null>
```

### Files to Modify

- `crates/mimir/frontend/src/composables/useTokens.ts`

### Dependencies

- MIMIR-T-0414 (Token Tauri commands)
- MIMIR-T-0415 (Token image serving)

## Status Updates

*To be added during implementation*
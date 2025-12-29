---
id: add-token-visibility-toggle-and
level: task
title: "Add token visibility toggle and quick-add in play mode"
short_code: "MIMIR-T-0210"
created_at: 2025-12-21T22:15:21.441922+00:00
updated_at: 2025-12-22T02:54:42.209956+00:00
parent: MIMIR-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0015
---

# Add token visibility toggle and quick-add in play mode

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Enable DMs to toggle token visibility (for ambushes, invisibility) and quickly add new tokens during play without leaving the session view.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Right-click token → Toggle Visibility option
- [ ] Keyboard shortcut (H) to hide/show selected token
- [ ] Visual indicator on DM view for hidden tokens
- [ ] Player display immediately hides/shows token
- [ ] Quick-add button in play mode sidebar
- [ ] Quick-add modal: select monster from catalog, place on map
- [ ] Quick-add respects current map and grid settings

## Implementation Notes

### Visibility Toggle

```typescript
async function toggleTokenVisibility(tokenId: number) {
  const result = await invoke('toggle_token_visibility', { id: tokenId })
  // Update local state
  // Emit IPC to player display
}
```

DM View indicators for hidden tokens:
- 50% opacity
- Dashed border or "eye-slash" icon overlay
- Tooltip: "Hidden from players"

### Quick-Add Flow

1. Click "+" button in play mode encounter sidebar
2. Modal opens with monster search (reuse catalog search component)
3. Select monster → shows on map at center with "placing" state
4. Click to place, or drag to position
5. Token created in database, appears on player display

```typescript
interface QuickAddState {
  isPlacing: boolean
  pendingToken: {
    monsterId: number
    monsterName: string
    size: TokenSize
  } | null
}
```

### Keyboard Shortcuts (Play Mode)
- `H` - Toggle visibility of selected token(s)
- `Delete` - Remove selected token(s)
- `Escape` - Cancel placement / deselect

### Files to Modify
- `crates/mimir-dm/frontend/src/features/modules/views/ModulePlayView.vue`
- `crates/mimir-dm/frontend/src/components/DmMapViewer.vue`
- `crates/mimir-dm/frontend/src/components/QuickAddTokenModal.vue` (new)

### Dependencies
- T-0208 (token rendering with visibility states)
- T-0206 (toggle_token_visibility command)
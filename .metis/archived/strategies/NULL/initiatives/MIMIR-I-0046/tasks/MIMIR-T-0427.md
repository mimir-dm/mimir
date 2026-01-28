---
id: token-context-menus
level: task
title: "Token context menus"
short_code: "MIMIR-T-0427"
created_at: 2026-01-25T02:44:32.943508+00:00
updated_at: 2026-01-25T16:04:45.341169+00:00
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

# Token context menus

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement right-click context menus for tokens with actions: Toggle Light, Toggle Dead, Toggle Visible.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Right-click on token shows context menu
- [ ] "Toggle Light" - adds/removes torch from token
- [ ] "Toggle Dead" - marks token dead/alive with visual indicator
- [ ] "Toggle Visible" - shows/hides from player display
- [ ] Menu closes on click outside or after action
- [ ] No keyboard shortcuts (context menu only)

## Implementation Notes

### Context Menu Options

| Action | Label | Effect | Runtime State |
|--------|-------|--------|---------------|
| Toggle Light | "Light Torch" / "Extinguish" | Add/remove light source attached to token | Runtime (play session) |
| Toggle Dead | "Mark Dead" / "Mark Alive" | Visual skull overlay, affects visibility | Runtime (play session) |
| Toggle Visible | "Hide from Players" / "Show to Players" | Token visibility for player display | Runtime (play session) |

### TokenContextMenu Component

```vue
<template>
  <div 
    v-if="visible"
    class="context-menu"
    :style="{ left: x + 'px', top: y + 'px' }"
    @click.stop
  >
    <div class="menu-item" @click="toggleLight">
      {{ hasLight ? 'Extinguish' : 'Light Torch' }}
    </div>
    <div class="menu-item" @click="toggleDead">
      {{ isDead ? 'Mark Alive' : 'Mark Dead' }}
    </div>
    <div class="menu-item" @click="toggleVisible">
      {{ isVisible ? 'Hide from Players' : 'Show to Players' }}
    </div>
  </div>
</template>

<script setup>
const emit = defineEmits(['toggle-light', 'toggle-dead', 'toggle-visible', 'close'])

function toggleLight() {
  emit('toggle-light')
  emit('close')
}
// ... other handlers
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: var(--surface-elevated);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.menu-item {
  padding: 8px 16px;
  cursor: pointer;
}

.menu-item:hover {
  background: var(--surface-hover);
}
</style>
```

### Integration in TokenRenderer

```vue
<template>
  <div 
    class="token"
    @contextmenu.prevent="showContextMenu"
  >
    <!-- Token content -->
    
    <TokenContextMenu
      v-if="contextMenuVisible"
      :x="contextMenuX"
      :y="contextMenuY"
      :has-light="hasAttachedLight"
      :is-dead="isDead"
      :is-visible="token.visible_to_players"
      @toggle-light="handleToggleLight"
      @toggle-dead="handleToggleDead"
      @toggle-visible="handleToggleVisible"
      @close="contextMenuVisible = false"
    />
  </div>
</template>
```

### Files to Create/Modify

- `crates/mimir/frontend/src/components/tokens/TokenContextMenu.vue`
- `crates/mimir/frontend/src/components/tokens/TokenRenderer.vue`

### Dependencies

- MIMIR-T-0418 (useTokens - for toggleDead, toggleVisible)
- MIMIR-T-0419 (useLightSources - for toggleLight)

## Status Updates

*To be added during implementation*
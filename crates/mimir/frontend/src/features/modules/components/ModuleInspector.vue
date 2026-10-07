<template>
  <aside
    v-if="selection"
    class="module-inspector"
    :class="{ collapsed }"
    :aria-label="`${KIND_LABEL[selection.kind]} details`"
    data-testid="module-inspector"
    @keydown.esc.stop="emit('close')"
  >
    <button
      type="button"
      class="inspector-toggle"
      :aria-expanded="!collapsed"
      :title="collapsed ? 'Show details' : 'Hide details'"
      data-testid="inspector-toggle"
      @click="collapsed = !collapsed"
    >
      <component :is="collapsed ? ChevronLeft : ChevronRight" :size="16" aria-hidden="true" />
    </button>
    <template v-if="!collapsed">
      <button
        type="button"
        class="inspector-close"
        title="Close"
        aria-label="Close details"
        data-testid="inspector-close"
        @click="emit('close')"
      >
        <X :size="18" aria-hidden="true" />
      </button>
      <MonsterStatsPanel v-if="selection.kind === 'monster'" :monster="selection.data" class="inspector-content" />
      <TrapDetailsPanel v-else-if="selection.kind === 'trap'" :trap="selection.data" class="inspector-content" />
      <PoiDetailsPanel v-else :poi="selection.data" class="inspector-content" />
    </template>
  </aside>
</template>

<script setup lang="ts">
/**
 * One details panel for the module dashboard (COLLIERY-I-0466, MIMIR-T-0693):
 * the frame (collapse, close, Esc) is shared, the content follows what is
 * selected — a monster, a trap or a point of interest.
 */
import { ref, watch } from 'vue'
import { ChevronLeft, ChevronRight, X } from '@lucide/vue'
import MonsterStatsPanel from './MonsterStatsPanel.vue'
import TrapDetailsPanel from './TrapDetailsPanel.vue'
import PoiDetailsPanel from './PoiDetailsPanel.vue'
import type { MonsterWithData } from '../composables/useModuleMonsters'
import type { ModulePoi, ModuleTrap } from '../types'

export type InspectorSelection =
  | { kind: 'monster'; data: MonsterWithData }
  | { kind: 'trap'; data: ModuleTrap }
  | { kind: 'poi'; data: ModulePoi }

const KIND_LABEL = { monster: 'Monster', trap: 'Trap', poi: 'Point of interest' } as const

const props = defineProps<{ selection: InspectorSelection | null }>()
const emit = defineEmits<{ close: [] }>()

const collapsed = ref(false)
// A new selection opens the panel again.
watch(
  () => props.selection,
  (now, before) => {
    if (now && now !== before) collapsed.value = false
  },
)
</script>

<style scoped>
.module-inspector {
  width: 380px;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  position: relative;
  transition: width 0.3s ease;
  overflow: hidden;
}

.module-inspector.collapsed {
  width: 32px;
}

.inspector-content {
  flex: 1;
  min-height: 0;
}

/* The content's own header leaves room for the close button. */
.module-inspector :deep(.inspector-content > header) {
  padding-right: 2.75rem;
}

.inspector-close {
  position: absolute;
  top: 0.6rem;
  right: 0.6rem;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.inspector-close:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
}

.inspector-toggle {
  position: absolute;
  left: -1px;
  top: 50%;
  transform: translateY(-50%);
  width: 24px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-right: none;
  border-radius: 6px 0 0 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  z-index: 10;
}

.inspector-toggle:hover {
  color: var(--color-text);
}
</style>

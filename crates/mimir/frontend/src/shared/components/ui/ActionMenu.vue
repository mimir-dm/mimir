<template>
  <div ref="root" class="action-menu" @keydown="onKeydown">
    <button
      ref="trigger"
      type="button"
      class="btn btn-secondary action-menu-trigger"
      aria-haspopup="menu"
      :aria-expanded="open"
      :aria-label="label"
      :title="label"
      data-testid="action-menu-trigger"
      @click="toggle"
    >
      <Ellipsis aria-hidden="true" :size="18" />
    </button>
    <ul v-if="open" class="action-menu-list" role="menu" :aria-label="label">
      <li v-for="(item, i) in items" :key="item.label" role="none">
        <button
          :ref="(el) => (itemEls[i] = el as HTMLButtonElement | null)"
          type="button"
          role="menuitem"
          tabindex="-1"
          :class="['action-menu-item', { danger: item.danger }]"
          data-testid="action-menu-item"
          @click="select(item)"
        >
          {{ item.label }}
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
/**
 * A "More actions" (…) button with a menu (COLLIERY-I-0466). Enter or Space
 * opens it with the first item focused; arrows move, Esc closes and gives the
 * focus back to the button; a click outside closes it.
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { Ellipsis } from '@lucide/vue'

export interface ActionMenuItem {
  label: string
  /** A destructive action: shown in red. */
  danger?: boolean
  onSelect: () => void
}

withDefaults(defineProps<{ items: ActionMenuItem[]; label?: string }>(), { label: 'More actions' })

const open = ref(false)
const root = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const itemEls = ref<(HTMLButtonElement | null)[]>([])

async function focusItem(index: number) {
  await nextTick()
  const els = itemEls.value.filter(Boolean) as HTMLButtonElement[]
  if (els.length === 0) return
  els[(index + els.length) % els.length].focus()
}

function toggle() {
  open.value = !open.value
  if (open.value) focusItem(0)
}

function close(returnFocus = true) {
  if (!open.value) return
  open.value = false
  if (returnFocus) trigger.value?.focus()
}

function select(item: ActionMenuItem) {
  close()
  item.onSelect()
}

function onKeydown(event: KeyboardEvent) {
  if (!open.value) return
  const els = itemEls.value.filter(Boolean) as HTMLButtonElement[]
  const current = els.indexOf(document.activeElement as HTMLButtonElement)
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    close()
  } else if (event.key === 'ArrowDown') {
    event.preventDefault()
    focusItem(current + 1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    focusItem(current - 1)
  } else if (event.key === 'Tab') {
    close(false)
  }
}

function onDocumentClick(event: MouseEvent) {
  if (open.value && root.value && !root.value.contains(event.target as Node)) close(false)
}

onMounted(() => document.addEventListener('click', onDocumentClick))
onUnmounted(() => document.removeEventListener('click', onDocumentClick))
</script>

<style scoped>
.action-menu {
  position: relative;
  display: inline-flex;
}

.action-menu-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.action-menu-list {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  z-index: 20;
  min-width: 10rem;
  margin: 0;
  padding: var(--spacing-xs) 0;
  list-style: none;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
}

.action-menu-item {
  display: block;
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  background: none;
  border: none;
  color: var(--color-text);
  text-align: left;
  font-size: 0.875rem;
  cursor: pointer;
}

.action-menu-item:hover,
.action-menu-item:focus-visible {
  background: var(--color-surface-hover);
  outline: none;
}

.action-menu-item.danger {
  color: var(--color-error);
}
</style>

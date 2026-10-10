<template>
  <div class="trap-panel trap-panel-content">
      <!-- Trap Header -->
      <header class="trap-header">
        <div class="trap-title">
          <h2>{{ trapData?.name || trap.name }}</h2>
          <p class="trap-type">{{ formatTrapType() }}</p>
        </div>
      </header>

      <div class="trap-body" v-if="trapData">
        <!-- Entries (trap description/mechanics) -->
        <div class="trap-entries">
          <div v-for="(entry, idx) in trapData.entries" :key="idx" class="entry-block">
            <template v-if="typeof entry === 'string'">
              <p v-html="processFormattingTags(entry)"></p>
            </template>
            <template v-else-if="entry.type === 'entries' || entry.entries">
              <div class="named-entry">
                <h4 v-if="entry.name">{{ entry.name }}</h4>
                <div v-for="(subEntry, subIdx) in (entry.entries || [])" :key="subIdx">
                  <p v-if="typeof subEntry === 'string'" v-html="processFormattingTags(subEntry)"></p>
                </div>
              </div>
            </template>
            <template v-else-if="entry.type === 'list'">
              <ul class="entry-list">
                <li v-for="(item, itemIdx) in entry.items" :key="itemIdx">
                  <span v-if="typeof item === 'string'" v-html="processFormattingTags(item)"></span>
                  <span v-else-if="item.name"><strong>{{ item.name }}.</strong>{{ ' ' }}
                    <span v-html="formatItemEntries(item)"></span>
                  </span>
                </li>
              </ul>
            </template>
            <template v-else-if="entry.type === 'table'">
              <table class="entry-table">
                <caption v-if="entry.caption">{{ entry.caption }}</caption>
                <thead v-if="entry.colLabels">
                  <tr>
                    <th v-for="(label, colIdx) in entry.colLabels" :key="colIdx">{{ label }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(row, rowIdx) in entry.rows" :key="rowIdx">
                    <td v-for="(cell, cellIdx) in row" :key="cellIdx">
                      {{ formatTableCell(cell) }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </template>
          </div>
        </div>
      </div>

      <!-- Loading state -->
      <div v-else-if="loading" class="trap-loading">
        Loading trap details...
      </div>

      <!-- Error/not found state -->
      <div v-else class="trap-not-found">
        <p>Trap "{{ trap.name }}" not found in catalog.</p>
        <p class="hint">Check that the trap name matches a catalog entry.</p>
      </div>

      <!-- Source -->
      <footer class="trap-footer" v-if="trapData">
        <span class="source-tag">{{ trapData.source }}</span>
      </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'

import type { ModuleTrap } from '../types'

interface TrapData {
  name: string
  source: string
  trapHazType?: string
  entries: any[]
}

const props = defineProps<{
  trap: ModuleTrap
}>()

const trapData = ref<TrapData | null>(null)
const loading = ref(false)

// Format trap type (Trap vs Hazard + mechanical/magical/etc.)
function formatTrapType(): string {
  if (!trapData.value) return 'Trap'

  const typeMap: Record<string, string> = {
    'MECH': 'Mechanical Trap',
    'MAG': 'Magical Trap',
    'WLD': 'Wilderness Hazard',
    'WTH': 'Weather Hazard',
    'ENV': 'Environmental Hazard'
  }

  return typeMap[trapData.value.trapHazType || ''] || 'Trap/Hazard'
}

// Format item entries (for list items with name + entry/entries)
function formatItemEntries(item: any): string {
  if (item.entry) {
    return typeof item.entry === 'string'
      ? processFormattingTags(item.entry)
      : ''
  }
  if (item.entries) {
    return item.entries
      .map((e: any) => typeof e === 'string' ? processFormattingTags(e) : '')
      .join(' ')
  }
  return ''
}

// Format table cell (can be string, number, or object)
function formatTableCell(cell: any): string {
  if (typeof cell === 'string') return cell
  if (typeof cell === 'number') return String(cell)
  if (cell.roll) {
    if (cell.roll.exact !== undefined) return String(cell.roll.exact)
    if (cell.roll.min !== undefined && cell.roll.max !== undefined) {
      return `${cell.roll.min}-${cell.roll.max}`
    }
  }
  return ''
}

// Load trap details from catalog
async function loadTrapDetails() {
  loading.value = true
  trapData.value = null
  try {
    const result = await invoke<{ success: boolean; data?: TrapData; error?: string }>('get_trap_by_name', {
      name: props.trap.name,
      source: props.trap.source
    })
    if (result.success && result.data) {
      trapData.value = result.data
    }
  } catch (e) {
    console.error('Failed to load trap details:', e)
  } finally {
    loading.value = false
  }
}

// Watch for trap changes
watch(() => props.trap, () => {
  loadTrapDetails()
}, { immediate: true })
</script>

<style scoped>

.trap-panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Trap Header */
.trap-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--color-surface);
  border-bottom: 2px solid var(--color-warning);
}

.trap-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1.2;
}

.trap-type {
  margin: 0.15rem 0 0 0;
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-secondary);
}

/* Trap Body */
.trap-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  text-align: left;
}

.trap-entries {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  text-align: left;
}

.entry-block p {
  margin: 0 0 0.5rem 0;
  font-size: 0.85rem;
  line-height: 1.5;
  color: var(--color-text);
}

.entry-block p:last-child {
  margin-bottom: 0;
}

/* Named entries (subsections) */
.named-entry {
  margin-bottom: 0.75rem;
}

.named-entry h4 {
  margin: 0 0 0.25rem 0;
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--color-warning);
}

/* Lists */
.entry-list {
  margin: 0;
  padding-left: 1.25rem;
  font-size: 0.85rem;
  line-height: 1.5;
}

.entry-list li {
  margin-bottom: 0.25rem;
}

/* Tables */
.entry-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
  margin: 0.5rem 0;
}

.entry-table caption {
  font-weight: 700;
  font-size: 0.85rem;
  text-align: left;
  margin-bottom: 0.25rem;
}

.entry-table th,
.entry-table td {
  padding: 0.35rem 0.5rem;
  border: 1px solid var(--color-border);
  text-align: left;
}

.entry-table th {
  background: var(--color-surface);
  font-weight: 600;
}

/* Loading state */
.trap-loading {
  padding: 1rem;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 0.85rem;
}

/* Not found state */
.trap-not-found {
  padding: 1rem;
  text-align: center;
}

.trap-not-found p {
  margin: 0 0 0.5rem 0;
  font-size: 0.85rem;
  color: var(--color-text);
}

.trap-not-found .hint {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Cross-ref styling within trap panel */
.trap-panel :deep(.cross-ref-link),
.trap-panel :deep(.spell-ref),
.trap-panel :deep(.item-ref),
.trap-panel :deep(.condition-ref) {
  color: var(--color-primary-text);
  text-decoration: underline;
  text-decoration-style: dotted;
  cursor: pointer;
}

.trap-panel :deep(.dice-roll),
.trap-panel :deep(.damage-roll) {
  font-family: monospace;
  font-weight: 700;
  color: var(--color-warning);
}

.trap-panel :deep(.hit-bonus) {
  font-weight: 700;
  color: var(--color-success);
}

/* Trap Footer */
.trap-footer {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.source-tag {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  font-style: italic;
}
</style>

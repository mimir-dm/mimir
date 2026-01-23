<template>
  <div class="tables-list-container">
    <table class="tables-list">
      <thead>
        <tr>
          <th @click="emit('sort', 'name')" class="sortable">
            Name
            <span class="sort-indicator" v-if="sortColumn === 'name'">
              {{ sortDirection === 'asc' ? '▲' : '▼' }}
            </span>
          </th>
          <th>Caption</th>
          <th @click="emit('sort', 'size')" class="sortable">
            Size
            <span class="sort-indicator" v-if="sortColumn === 'size'">
              {{ sortDirection === 'asc' ? '▲' : '▼' }}
            </span>
          </th>
          <th @click="emit('sort', 'source')" class="sortable">
            Source
            <span class="sort-indicator" v-if="sortColumn === 'source'">
              {{ sortDirection === 'asc' ? '▲' : '▼' }}
            </span>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="table in tables" :key="`${table.name}-${table.source}`"
            @click="emit('select', table)" class="clickable-row">
          <td class="name-cell">{{ table.name }}</td>
          <td class="caption-cell">{{ table.caption || '—' }}</td>
          <td class="size-cell">
            <span class="size-info">
              {{ getColumnCount(table) }}×{{ getRowCount(table) }}
            </span>
          </td>
          <td class="source-cell">{{ table.source }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { TableSummary } from '../../composables/catalog'

defineProps<{
  tables: TableSummary[]
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [table: TableSummary]
}>()

// Get column count from colLabels array (5etools format)
function getColumnCount(table: any): number {
  if (Array.isArray(table.colLabels)) {
    return table.colLabels.length
  }
  return 0
}

// Get row count from rows array (5etools format)
function getRowCount(table: any): number {
  if (Array.isArray(table.rows)) {
    return table.rows.length
  }
  return 0
}
</script>

<style scoped>
.tables-list-container {
  width: 100%;
  overflow-x: auto;
}

.tables-list {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.tables-list th {
  text-align: left;
  padding: var(--spacing-sm, 8px);
  border-bottom: 2px solid var(--color-border, #333);
  background: var(--color-surface, #1a1a1a);
  color: var(--color-text-secondary, #999);
  white-space: nowrap;
}

.tables-list th.sortable {
  cursor: pointer;
  user-select: none;
}

.tables-list th.sortable:hover {
  color: var(--color-text, #e0e0e0);
}

.sort-indicator {
  display: inline-block;
  margin-left: 4px;
  font-size: 0.8em;
}

.tables-list tbody tr {
  border-bottom: 1px solid var(--color-border-light, #262626);
  transition: background-color 0.15s ease;
}

.tables-list tbody tr:hover {
  background: var(--color-surface-hover, #262626);
}

.clickable-row {
  cursor: pointer;
}

.tables-list td {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  color: var(--color-text, #e0e0e0);
}

/* Cell-specific styles */
.name-cell {
  font-weight: 500;
  color: var(--color-primary, #4a9eff);
}

.caption-cell {
  max-width: 300px;
  font-size: 0.85rem;
  color: var(--color-text-secondary, #999);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.size-cell {
  white-space: nowrap;
  text-align: center;
}

.size-info {
  font-family: monospace;
  font-size: 0.85rem;
  color: var(--color-accent, #ff6b6b);
}

.source-cell {
  color: var(--color-text-secondary, #999);
  font-size: 0.85rem;
  white-space: nowrap;
}
</style>
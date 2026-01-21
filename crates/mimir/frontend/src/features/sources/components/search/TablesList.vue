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
          <th @click="emit('sort', 'category')" class="sortable">
            Category
            <span class="sort-indicator" v-if="sortColumn === 'category'">
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
          <td class="category-cell">
            <span :class="['category-badge', getCategoryClass(table.category)]">
              {{ table.category }}
            </span>
          </td>
          <td class="caption-cell">{{ table.caption }}</td>
          <td class="size-cell">
            <span class="size-info">
              {{ table.columns }}×{{ table.rows }}
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

function getCategoryClass(category: string | undefined): string {
  if (!category) return 'category-general'
  switch (category.toLowerCase()) {
    case 'madness':
      return 'category-madness'
    case 'treasure':
      return 'category-treasure'
    case 'encounters':
      return 'category-encounters'
    case 'trinkets':
      return 'category-trinkets'
    case 'wild magic':
      return 'category-wild-magic'
    case 'combat':
      return 'category-combat'
    case 'npcs':
      return 'category-npcs'
    case 'adventures':
      return 'category-adventures'
    case 'magic items':
      return 'category-magic-items'
    default:
      return 'category-misc'
  }
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

.category-cell {
  white-space: nowrap;
}

.category-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: 500;
}

.category-madness {
  background: rgba(156, 39, 176, 0.2);
  color: #9c27b0;
  border: 1px solid rgba(156, 39, 176, 0.4);
}

.category-treasure {
  background: rgba(255, 193, 7, 0.2);
  color: #ffc107;
  border: 1px solid rgba(255, 193, 7, 0.4);
}

.category-encounters {
  background: rgba(244, 67, 54, 0.2);
  color: #f44336;
  border: 1px solid rgba(244, 67, 54, 0.4);
}

.category-trinkets {
  background: rgba(0, 188, 212, 0.2);
  color: #00bcd4;
  border: 1px solid rgba(0, 188, 212, 0.4);
}

.category-wild-magic {
  background: rgba(103, 58, 183, 0.2);
  color: #673ab7;
  border: 1px solid rgba(103, 58, 183, 0.4);
}

.category-combat {
  background: rgba(255, 87, 34, 0.2);
  color: #ff5722;
  border: 1px solid rgba(255, 87, 34, 0.4);
}

.category-npcs {
  background: rgba(76, 175, 80, 0.2);
  color: #4caf50;
  border: 1px solid rgba(76, 175, 80, 0.4);
}

.category-adventures {
  background: rgba(33, 150, 243, 0.2);
  color: #2196f3;
  border: 1px solid rgba(33, 150, 243, 0.4);
}

.category-magic-items {
  background: rgba(255, 152, 0, 0.2);
  color: #ff9800;
  border: 1px solid rgba(255, 152, 0, 0.4);
}

.category-misc {
  background: var(--color-surface, #1a1a1a);
  color: var(--color-text-secondary, #999);
  border: 1px solid var(--color-border, #333);
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
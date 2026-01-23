import type { CatalogConfig } from './types'

// Extract description from entries (5etools format)
function formatDescription(_value: unknown, item: unknown): string {
  if (typeof item !== 'object' || item === null) return '—'

  const record = item as Record<string, unknown>
  const entries = record.entries

  if (!Array.isArray(entries) || entries.length === 0) return '—'

  // Look for text content in entries
  for (const entry of entries) {
    // Direct string entry
    if (typeof entry === 'string') {
      return truncateText(entry, 100)
    }

    // List entry with items
    if (typeof entry === 'object' && entry !== null) {
      const entryObj = entry as Record<string, unknown>

      // Handle list type with items
      if (entryObj.type === 'list' && Array.isArray(entryObj.items)) {
        const firstItem = entryObj.items[0]
        if (typeof firstItem === 'string') {
          return truncateText(firstItem, 100)
        }
      }

      // Handle entries property
      if (typeof entryObj.entries === 'string') {
        return truncateText(entryObj.entries, 100)
      }
    }
  }

  return '—'
}

// Truncate text with ellipsis
function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength).trim() + '...'
}

export const conditionConfig: CatalogConfig = {
  name: 'conditions',
  title: 'Conditions & Diseases',
  searchCommands: {
    search: 'search_conditions',
    details: 'get_condition',
    sources: 'get_condition_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'item_type',
      label: 'Type',
      type: 'text',
      className: 'catalog-table__cell-center'
    },
    {
      key: 'entries',
      label: 'Description',
      type: 'text',
      className: 'catalog-table__cell-description',
      formatter: formatDescription
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source'
    }
  ],
  filters: [
    {
      type: 'text',
      key: 'search',
      label: 'Search',
      placeholder: 'Search conditions & diseases...'
    },
    {
      key: 'item_type',
      type: 'checkbox-group',
      label: 'Type:',
      options: [
        { value: 'Condition', label: 'Condition' },
        { value: 'Disease', label: 'Disease' }
      ]
    }
  ]
}
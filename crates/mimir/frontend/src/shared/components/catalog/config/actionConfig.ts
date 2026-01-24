import type { CatalogConfig } from './types'

// Format time from 5etools format (array of objects or strings)
function formatTime(time: unknown): string {
  if (!Array.isArray(time) || time.length === 0) return '—'

  const formatted = time.map((t: unknown) => {
    // Handle string format (e.g., "Varies")
    if (typeof t === 'string') return t

    // Handle object format (e.g., {"number": 1, "unit": "action"})
    if (typeof t === 'object' && t !== null) {
      const timeObj = t as Record<string, unknown>
      const num = timeObj.number
      const unit = timeObj.unit

      if (typeof unit === 'string') {
        if (typeof num === 'number') {
          // Pluralize if needed
          const unitStr = num === 1 ? unit : `${unit}s`
          return `${num} ${unitStr}`
        }
        return unit
      }
    }
    return null
  }).filter(Boolean)

  return formatted.length > 0 ? formatted.join(' or ') : '—'
}

export const actionConfig: CatalogConfig = {
  name: 'actions',
  title: 'Actions',
  searchCommands: {
    search: 'search_actions',
    details: 'get_action',
    sources: 'get_action_sources',
    timeTypes: 'get_action_time_types'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'time',
      label: 'Time',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatTime
    },
    {
      key: 'description',
      label: 'Description',
      type: 'text',
      className: 'catalog-table__cell-description'
    },
    {
      key: 'see_also',
      label: 'See Also',
      type: 'array',
      formatter: (action: any) => {
        if (!action || !action.see_also || action.see_also.length === 0) {
          return '—'
        }
        return action.see_also.join(', ')
      },
      className: 'catalog-table__cell-secondary'
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
      placeholder: 'Search actions...'
    },
    {
      type: 'checkbox-group',
      key: 'time_types',
      label: 'Time:',
      options: [
        { value: 'action', label: 'Action' },
        { value: 'bonus', label: 'Bonus' },
        { value: 'reaction', label: 'Reaction' },
        { value: 'free', label: 'Free' },
        { value: 'varies', label: 'Varies' }
      ]
    }
  ]
}
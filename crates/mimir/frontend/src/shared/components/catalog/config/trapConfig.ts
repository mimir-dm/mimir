import type { CatalogConfig } from './types'

export const trapConfig: CatalogConfig = {
  name: 'traps',
  title: 'Traps & Hazards',
  searchCommands: {
    search: 'search_traps',
    details: 'get_trap_details',
    sources: 'get_trap_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-name'
    },
    {
      key: 'category',
      label: 'Category',
      type: 'badge',
      sortable: true,
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        const value = item.category || '—'
        let variant = 'default'
        switch (value) {
          case 'Trap':
            variant = 'trap'
            break
          case 'Hazard':
            variant = 'hazard'
            break
        }
        return { text: value, variant }
      }
    },
    {
      key: 'trap_type',
      label: 'Type',
      type: 'text',
      sortable: true,
      className: 'catalog-table__cell-center',
      formatter: (value: unknown) => {
        if (!value || value === '—') return '—'
        return String(value)
      }
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
      key: 'category',
      type: 'checkbox-group',
      label: 'Category:',
      options: [
        { value: 'Trap', label: 'Trap' },
        { value: 'Hazard', label: 'Hazard' }
      ]
    }
  ],
  emptyMessage: {
    title: 'No traps or hazards found',
    subtitle: 'Search for traps and hazards to see results',
    noResults: 'No traps or hazards found matching your criteria'
  }
}
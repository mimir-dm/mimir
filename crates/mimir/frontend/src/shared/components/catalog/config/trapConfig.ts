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
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        // Return badge configuration for CatalogTable to handle
        const value = item.category
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
      className: 'catalog-table__cell-center'
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source',
      formatter: (value: string) => value
    }
  ],
  filters: [],
  emptyMessage: {
    title: 'No traps or hazards found',
    subtitle: 'Search for traps and hazards to see results',
    noResults: 'No traps or hazards found matching your criteria'
  }
}
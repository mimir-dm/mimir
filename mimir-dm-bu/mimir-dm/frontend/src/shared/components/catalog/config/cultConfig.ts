import type { CatalogConfig } from './types'

export const cultConfig: CatalogConfig = {
  name: 'cults',
  title: 'Cults & Boons',
  searchCommands: {
    search: 'search_cults',
    details: 'get_cult_details',
    sources: 'get_cult_sources'
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
      key: 'item_type',
      label: 'Category',
      sortable: true,
      type: 'badge',
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        const value = item.item_type
        let variant = 'default'
        switch (value) {
          case 'cult':
            variant = 'cult'
            break
          case 'boon':
            variant = 'boon'
            break
        }
        const displayValue = value === 'cult' ? 'Cult' : 'Boon'
        return { text: displayValue, variant }
      }
    },
    {
      key: 'subtype',
      label: 'Type',
      sortable: true,
      type: 'badge',
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        if (!item.subtype) {
          return { text: '—', variant: 'default' }
        }
        
        const value = item.subtype
        let variant = 'default'
        switch (value.toLowerCase()) {
          case 'diabolical':
            variant = 'diabolical'
            break
          case 'demonic':
            variant = 'demonic'
            break
          case 'elder evil':
            variant = 'elder-evil'
            break
          default:
            variant = 'other'
        }
        return { text: value, variant }
      }
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source',
      formatter: (value: string) => value
    },
    {
      key: 'page',
      label: 'Page',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: (item: any) => (item && item.page) ? `p. ${item.page}` : '—'
    }
  ],
  filters: [],
  emptyMessage: {
    title: 'No cults or boons found',
    subtitle: 'Search for cults and boons to see results',
    noResults: 'No cults or boons found matching your criteria'
  }
}
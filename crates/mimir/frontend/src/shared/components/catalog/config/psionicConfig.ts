import type { CatalogConfig } from './types'

export const psionicConfig: CatalogConfig = {
  name: 'psionic',
  title: 'Psionics',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__name'
    },
    {
      key: 'psionic_type',
      label: 'Type',
      sortable: true,
      type: 'badge',
      formatter: (value: string) => {
        const display = value === 'D' ? 'Discipline' : 'Talent'
        const variant = value === 'D' ? 'discipline' : 'talent'
        return { text: display, variant }
      }
    },
    {
      key: 'order',
      label: 'Order',
      sortable: true,
      type: 'badge',
      formatter: (value: string | null | undefined) => {
        if (!value || typeof value !== 'string') {
          return '—'
        }
        const variant = `order-${value.toLowerCase().replace(/\s+/g, '-')}`
        return { text: value, variant }
      }
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'source',
      formatter: (value: any) => {
        if (typeof value === 'object' && value.source) {
          return { source: value.source, showSrd: false }
        }
        return { source: value, showSrd: false }
      }
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'psionic_types',
      label: 'Type',
      options: [
        { value: 'D', label: 'Discipline' },
        { value: 'T', label: 'Talent' }
      ]
    },
    {
      type: 'multiselect',
      key: 'orders',
      label: 'Order',
      apiSource: 'get_psionic_orders'
    },
    {
      type: 'multiselect',
      key: 'sources',
      label: 'Source',
      apiSource: 'get_psionic_sources'
    }
  ],
  searchCommands: {
    search: 'search_psionics',
    details: 'get_psionic_details'
  }
}
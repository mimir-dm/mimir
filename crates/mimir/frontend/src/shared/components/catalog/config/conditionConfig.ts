import type { CatalogConfig } from './types'

export const conditionConfig: CatalogConfig = {
  name: 'conditions',
  title: 'Conditions & Diseases',
  searchCommands: {
    search: 'search_conditions',
    details: 'get_condition',
    sources: 'get_condition_sources',
    itemTypes: 'get_condition_item_types'
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
      type: 'badge',
      className: 'catalog-table__cell-center',
      formatter: (condition: any) => {
        return {
          text: condition.item_type || 'Unknown',
          variant: condition.item_type?.toLowerCase() || 'unknown'
        }
      }
    },
    {
      key: 'description',
      label: 'Description',
      type: 'text',
      className: 'catalog-table__cell-description'
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
      placeholder: 'Search conditions...'
    },
    {
      type: 'multiselect',
      key: 'item_types',
      label: 'Type',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_condition_item_types'
    }
  ]
}
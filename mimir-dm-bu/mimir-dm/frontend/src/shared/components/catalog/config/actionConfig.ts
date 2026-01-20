import type { CatalogConfig } from './types'

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
      className: 'catalog-table__cell-center'
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
      type: 'multiselect',
      key: 'time_types',
      label: 'Time',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_action_time_types'
    },
    {
      type: 'multiselect',
      key: 'sources',
      label: 'Source',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_action_sources'
    }
  ]
}
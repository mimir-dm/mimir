import type { CatalogConfig } from './types'

export const featConfig: CatalogConfig = {
  name: 'feats',
  title: 'Feats',
  searchCommands: {
    search: 'search_feats',
    details: 'get_feat_details',
    sources: 'get_feat_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'prerequisites',
      label: 'Prerequisites',
      type: 'text',
      className: 'catalog-table__cell-prerequisites',
      formatter: (value: string | null) => value || '—'
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
      placeholder: 'Search feats...'
    },
    {
      type: 'multiselect',
      key: 'sources',
      label: 'Source',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_feat_sources'
    },
    {
      type: 'checkbox',
      key: 'has_prerequisites',
      label: 'Has Prerequisites',
      tooltip: 'Show only feats that have prerequisites'
    }
  ],
  emptyMessage: {
    title: 'No feats found',
    subtitle: 'Search for feats to see results',
    noResults: 'No feats found matching your criteria'
  }
}
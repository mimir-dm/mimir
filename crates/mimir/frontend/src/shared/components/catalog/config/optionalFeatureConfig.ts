import type { CatalogConfig } from './types'

export const optionalFeatureConfig: CatalogConfig = {
  name: 'optionalFeatures',
  title: 'Optional Features',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'feature_type_full',
      label: 'Type',
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    },
    {
      key: 'prerequisite_text',
      label: 'Prerequisites',
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value || '—'
      }
    },
    {
      key: 'source',
      label: 'Source',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'feature_types',
      label: 'Feature Type',
      apiSource: 'get_optional_feature_types'
    },
    {
      type: 'checkbox',
      key: 'grants_spells',
      label: 'Grants Spells',
      tooltip: 'Filter by features that grant additional spells'
    }
  ],
  searchCommands: {
    search: 'search_optional_features',
    details: 'get_optional_feature_details',
    sources: 'get_optional_feature_sources'
  },
  emptyMessage: {
    title: 'No optional features found',
    subtitle: 'Search for optional features and class variants',
    noResults: 'No optional features found matching your criteria'
  },
  searchPlaceholder: 'Search optional features...'
}
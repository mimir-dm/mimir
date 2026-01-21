import type { CatalogConfig } from './types'

export const backgroundConfig: CatalogConfig = {
  name: 'backgrounds',
  title: 'Backgrounds',
  searchCommands: {
    search: 'search_backgrounds',
    details: 'get_background_details',
    sources: 'get_background_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'skills',
      label: 'Skills',
      type: 'text',
      className: 'catalog-table__cell-skills'
    },
    {
      key: 'languages',
      label: 'Languages', 
      type: 'text',
      className: 'catalog-table__cell-languages'
    },
    {
      key: 'tools',
      label: 'Tools',
      type: 'text',
      className: 'catalog-table__cell-tools'
    },
    {
      key: 'feature',
      label: 'Feature',
      type: 'text',
      className: 'catalog-table__cell-feature'
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
      placeholder: 'Search backgrounds...'
    },
    {
      type: 'multiselect',
      key: 'sources',
      label: 'Source',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_background_sources'
    },
    {
      type: 'checkbox',
      key: 'has_tools',
      label: 'Has Tools',
      tooltip: 'Show only backgrounds that provide tool proficiencies'
    }
  ],
  emptyMessage: {
    title: 'No backgrounds found',
    subtitle: 'Search for backgrounds to see results',
    noResults: 'No backgrounds found matching your criteria'
  }
}
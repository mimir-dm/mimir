import type { CatalogConfig } from './types'

export const languageConfig: CatalogConfig = {
  name: 'languages',
  title: 'Languages',
  searchCommands: {
    search: 'search_languages',
    details: 'get_language_details',
    sources: 'get_language_sources',
    itemTypes: 'get_language_types'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'language_type',
      label: 'Type',
      type: 'badge',
      className: 'catalog-table__cell-center',
      formatter: (language: any) => {
        const typeClass = getTypeClass(language.language_type);
        return {
          text: language.language_type || 'Standard',
          variant: typeClass
        }
      }
    },
    {
      key: 'script',
      label: 'Script',
      type: 'text',
      className: 'catalog-table__cell-center'
    },
    {
      key: 'typical_speakers',
      label: 'Typical Speakers',
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
      placeholder: 'Search languages...'
    },
    {
      type: 'multiselect',
      key: 'language_types',
      label: 'Type',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_language_types'
    },
    {
      type: 'multiselect',
      key: 'scripts',
      label: 'Script',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_language_scripts'
    }
  ]
}

function getTypeClass(type: string): string {
  if (!type) return 'type-standard'
  
  switch (type.toLowerCase()) {
    case 'standard':
      return 'type-standard'
    case 'exotic':
      return 'type-exotic'
    case 'secret':
      return 'type-secret'
    case 'dead':
      return 'type-dead'
    case 'primordial dialect':
      return 'type-primordial'
    default:
      return 'type-default'
  }
}
import type { CatalogConfig } from './types'

// Format domains array for display
function formatDomains(domains: string[] | null): string {
  if (!domains || domains.length === 0) return '—'
  return domains.join(', ')
}

// Format alignment for display
function formatAlignment(alignment: string | null): string {
  if (!alignment) return '—'
  return alignment
}

// Format title for display
function formatTitle(title: string | null): string {
  if (!title) return '—'
  return title
}

export const deityConfig: CatalogConfig = {
  name: 'deities',
  title: 'Deities',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'title',
      label: 'Title',
      type: 'text',
      formatter: (value: any) => {
        return formatTitle(typeof value === 'string' ? value : null)
      }
    },
    {
      key: 'pantheon',
      label: 'Pantheon',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    },
    {
      key: 'alignment',
      label: 'Alignment',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        return formatAlignment(typeof value === 'string' ? value : null)
      }
    },
    {
      key: 'domains',
      label: 'Domains',
      type: 'array',
      formatter: (value: any) => {
        return formatDomains(Array.isArray(value) ? value : null)
      }
    },
    {
      key: 'symbol',
      label: 'Symbol',
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
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
      key: 'pantheons',
      label: 'Pantheon',
      apiSource: 'get_deity_pantheons'
    },
    {
      type: 'multiselect',
      key: 'alignments',
      label: 'Alignment',
      apiSource: 'get_deity_alignments'
    },
    {
      type: 'multiselect',
      key: 'domains',
      label: 'Domains',
      apiSource: 'get_deity_domains'
    }
  ],
  searchCommands: {
    search: 'search_deities',
    details: 'get_deity_details',
    sources: 'get_deity_statistics'
  },
  emptyMessage: {
    title: 'No deities found',
    subtitle: 'Search for gods, goddesses, and divine beings',
    noResults: 'No deities found matching your criteria'
  },
  searchPlaceholder: 'Search deities...'
}
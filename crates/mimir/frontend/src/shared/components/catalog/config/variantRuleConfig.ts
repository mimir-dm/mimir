import type { CatalogConfig } from './types'

export const variantRuleConfig: CatalogConfig = {
  name: 'variant-rule',
  title: 'Variant Rules',
  columns: [
    { 
      key: 'name', 
      label: 'Name', 
      sortable: true 
    },
    { 
      key: 'rule_type', 
      label: 'Type', 
      type: 'badge',
      formatter: (value: any) => {
        const text = (!value || typeof value !== 'string') ? 'General' : value
        const variant = text.toLowerCase().replace(/\s+/g, '-')
        return { text, variant }
      },
      sortable: true 
    },
    { 
      key: 'source', 
      label: 'Source', 
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      },
      sortable: true 
    },
    { 
      key: 'page', 
      label: 'Page',
      formatter: (value: any) => {
        if (!value || typeof value !== 'number') return '—'
        return `p. ${value}`
      }
    }
  ],
  filters: [
    {
      key: 'query',
      type: 'text',
      label: 'Search variant rules...',
      placeholder: 'Enter name...'
    },
    {
      key: 'rule_types',
      type: 'multiselect',
      label: 'Types',
      placeholder: 'Select types...',
      apiSource: 'get_variant_rule_types'
    },
    {
      key: 'sources',
      type: 'multiselect', 
      label: 'Sources',
      placeholder: 'Select sources...',
      apiSource: 'get_variant_rule_sources'
    }
  ],
  searchCommands: {
    search: 'search_variant_rules',
    details: 'get_variant_rule'
  },
  emptyMessage: {
    title: 'No variant rules found',
    subtitle: 'Try adjusting your search criteria',
    noResults: 'No variant rules match your current filters'
  }
}
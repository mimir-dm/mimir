import type { CatalogConfig } from './types'

// Format rule type from 5etools format
// Note: badge type columns receive the full item, not just the field value
function formatRuleType(item: unknown): { text: string; variant: string } {
  const ruleType = (item as any)?.ruleType
  if (typeof ruleType === 'string') {
    const typeMap: Record<string, string> = {
      'C': 'Core',
      'O': 'Optional',
      'V': 'Variant',
      'VO': 'Variant Optional',
      'OV': 'Optional Variant'
    }
    const text = typeMap[ruleType] || ruleType
    return { text, variant: ruleType.toLowerCase() }
  }
  return { text: '—', variant: 'none' }
}

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
      key: 'ruleType',
      label: 'Type',
      type: 'badge',
      formatter: formatRuleType,
      sortable: true
    },
    {
      key: 'source',
      label: 'Source',
      type: 'text',
      sortable: true
    },
    {
      key: 'page',
      label: 'Page',
      formatter: (value: unknown) => {
        if (typeof value === 'number') return `p. ${value}`
        return '—'
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
      key: 'ruleType',
      type: 'checkbox-group',
      label: 'Type:',
      options: [
        { value: 'C', label: 'Core' },
        { value: 'O', label: 'Optional' },
        { value: 'V', label: 'Variant' },
        { value: 'VO', label: 'Var. Opt.' }
      ]
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

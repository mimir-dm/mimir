import type { CatalogConfig } from './types'
import { formatWeight } from '../../../utils/formatters'

// Format cost from copper pieces to appropriate currency
function formatCost(value: number | null): string {
  if (!value || value === 0) return '—'
  
  if (value >= 100) {
    return `${value / 100} gp`
  } else if (value >= 10) {
    return `${value / 10} sp`
  } else {
    return `${value} cp`
  }
}

// Format rarity for display
function formatRarity(rarity: string | null): string {
  if (!rarity || rarity === 'none') return '—'
  return rarity.charAt(0).toUpperCase() + rarity.slice(1)
}

export const itemConfig: CatalogConfig = {
  name: 'items',
  title: 'Items',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'typeName',
      label: 'Type',
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    },
    {
      key: 'value',
      label: 'Cost',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        return formatCost(typeof value === 'number' ? value : null)
      }
    },
    {
      key: 'weight',
      label: 'Weight',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        return formatWeight(typeof value === 'number' ? value : null)
      }
    },
    {
      key: 'rarity',
      label: 'Rarity',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        return formatRarity(typeof value === 'string' ? value : null)
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
      key: 'item_types',
      label: 'Item Type',
      apiSource: 'get_item_types'
    },
    {
      type: 'multiselect',
      key: 'rarities',
      label: 'Rarity',
      apiSource: 'get_item_rarities'
    },
    {
      type: 'range',
      key: 'value_range',
      label: 'Cost (gp)',
      min: 0,
      max: 100000,
      step: 1
    }
  ],
  searchCommands: {
    search: 'search_items',
    details: 'get_item_details',
    sources: 'get_item_sources'
  },
  emptyMessage: {
    title: 'No items found',
    subtitle: 'Search for equipment, weapons, armor, and magical items',
    noResults: 'No items found matching your criteria'
  },
  searchPlaceholder: 'Search items...'
}
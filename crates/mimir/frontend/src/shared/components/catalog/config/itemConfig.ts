import type { CatalogConfig } from './types'
import { formatWeight } from '../../../utils/formatters'

// Item type code to name mapping
const itemTypeMap: Record<string, string> = {
  'A': 'Armor',
  'AF': 'Ammunition',
  'AIR': 'Vehicle (Air)',
  'AT': 'Artisan\'s Tools',
  'EM': 'Eldritch Machine',
  'EXP': 'Explosive',
  'FD': 'Food and Drink',
  'G': 'Adventuring Gear',
  'GS': 'Gaming Set',
  'GV': 'Generic Variant',
  'HA': 'Heavy Armor',
  'INS': 'Instrument',
  'LA': 'Light Armor',
  'M': 'Melee Weapon',
  'MA': 'Medium Armor',
  'MNT': 'Mount',
  'MR': 'Master Rune',
  'OTH': 'Other',
  'P': 'Potion',
  'R': 'Ranged Weapon',
  'RD': 'Rod',
  'RG': 'Ring',
  'S': 'Shield',
  'SC': 'Scroll',
  'SCF': 'Spellcasting Focus',
  'SHP': 'Vehicle (Water)',
  'T': 'Tools',
  'TAH': 'Tack and Harness',
  'TG': 'Trade Good',
  'VEH': 'Vehicle (Land)',
  'WD': 'Wand',
  '$': 'Treasure',
  '$A': 'Treasure (Art)',
  '$C': 'Treasure (Coinage)',
  '$G': 'Treasure (Gemstone)',
}

// Format item type - handles string type codes from 5etools
// Type can be "G" or "G|XPHB" (with source suffix) - strip the source part
function formatItemType(value: unknown): string {
  if (typeof value === 'string') {
    // Strip source suffix if present (e.g., "G|XPHB" -> "G")
    const typeCode = value.split('|')[0]
    return itemTypeMap[typeCode] || typeCode
  }
  return '—'
}

// Format cost from 5etools value field (in copper pieces) to appropriate currency
function formatCost(value: unknown): string {
  const numValue = typeof value === 'number' ? value : null
  if (!numValue || numValue === 0) return '—'

  if (numValue >= 100) {
    return `${numValue / 100} gp`
  } else if (numValue >= 10) {
    return `${numValue / 10} sp`
  } else {
    return `${numValue} cp`
  }
}

// Format rarity for display
function formatRarity(rarity: unknown): string {
  if (!rarity || rarity === 'none') return '—'
  if (typeof rarity === 'string') {
    return rarity.charAt(0).toUpperCase() + rarity.slice(1)
  }
  return '—'
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
      key: 'type',
      label: 'Type',
      type: 'text',
      formatter: formatItemType
    },
    {
      key: 'value',
      label: 'Cost',
      type: 'text',
      sortable: true,
      formatter: formatCost
    },
    {
      key: 'weight',
      label: 'Weight',
      type: 'text',
      sortable: true,
      formatter: (value: unknown) => formatWeight(typeof value === 'number' ? value : null)
    },
    {
      key: 'rarity',
      label: 'Rarity',
      type: 'text',
      sortable: true,
      formatter: formatRarity
    },
    {
      key: 'source',
      label: 'Source',
      type: 'text',
      sortable: true
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
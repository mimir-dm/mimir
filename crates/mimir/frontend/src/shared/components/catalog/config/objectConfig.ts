import type { CatalogConfig } from './types'

// Object type code to name mapping
const objectTypeMap: Record<string, string> = {
  'SW': 'Siege Weapon',
  'GEN': 'Generic',
  'U': 'Unknown',
  'generic': 'Generic',
}

// Size code to name mapping
const sizeMap: Record<string, string> = {
  'T': 'Tiny',
  'S': 'Small',
  'M': 'Medium',
  'L': 'Large',
  'H': 'Huge',
  'G': 'Gargantuan',
}

// Format object type from code or value
function formatObjectType(value: unknown): string {
  if (!value) return '—'
  if (typeof value === 'string') {
    return objectTypeMap[value] || value
  }
  return '—'
}

// Format size - handles both single values and arrays
function formatSize(value: unknown): string {
  if (!value) return '—'
  if (Array.isArray(value)) {
    return value.map(s => sizeMap[s] || s).join('/')
  }
  if (typeof value === 'string') {
    return sizeMap[value] || value
  }
  return '—'
}

// Format AC - can be number or object with special properties
function formatAC(value: unknown): string {
  if (value === undefined || value === null) return '—'
  if (typeof value === 'number') return value.toString()
  if (typeof value === 'object' && value !== null) {
    const ac = value as Record<string, unknown>
    if (ac.special) return String(ac.special)
    if (ac.ac !== undefined) return String(ac.ac)
  }
  return '—'
}

// Format HP - can be number, string, or object with average/formula
function formatHP(value: unknown): string {
  if (value === undefined || value === null) return '—'
  if (typeof value === 'number') return value.toString()
  if (typeof value === 'string') return value
  if (typeof value === 'object' && value !== null) {
    const hp = value as Record<string, unknown>
    // Handle {average: X, formula: "XdY+Z"} format
    if (hp.average !== undefined) {
      const avg = hp.average
      const formula = hp.formula
      if (formula) {
        return `${avg} (${formula})`
      }
      return String(avg)
    }
    // Handle {special: "..."} format
    if (hp.special) return String(hp.special)
  }
  return '—'
}

export const objectConfig: CatalogConfig = {
  name: 'objects',
  title: 'Objects',
  searchCommands: {
    search: 'search_objects',
    details: 'get_object_details',
    sources: 'get_object_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'objectType',
      label: 'Type',
      type: 'badge',
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        // Get type from objectType field (camelCase from parsed JSON)
        const rawType = item.objectType || item.object_type
        const displayType = formatObjectType(rawType)
        let variant = 'default'
        switch (displayType) {
          case 'Siege Weapon':
            variant = 'siege'
            break
          case 'Generic':
            variant = 'generic'
            break
        }
        return { text: displayType || '—', variant }
      }
    },
    {
      key: 'size',
      label: 'Size',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatSize
    },
    {
      key: 'ac',
      label: 'AC',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatAC
    },
    {
      key: 'hp',
      label: 'HP',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatHP
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
      type: 'checkbox-group',
      key: 'object_type',
      label: 'Type:',
      options: [
        { value: 'SW', label: 'Siege' },
        { value: 'GEN', label: 'Generic' },
        { value: 'U', label: 'Unknown' }
      ]
    },
    {
      type: 'checkbox-group',
      key: 'sizes',
      label: 'Size:',
      options: [
        { value: 'T', label: 'Tiny' },
        { value: 'S', label: 'Small' },
        { value: 'M', label: 'Med' },
        { value: 'L', label: 'Large' },
        { value: 'H', label: 'Huge' },
        { value: 'G', label: 'Garg' }
      ]
    }
  ],
  emptyMessage: {
    title: 'No objects found',
    subtitle: 'Search for objects to see results',
    noResults: 'No objects found matching your criteria'
  }
}